//! Frozen-script diagnostics through the existing grounded session.
//!
//! Only source text, caller scope changes and explicit read/write controls
//! reach the session. Labels, expected memory and complete answer forms are
//! scoring data. This module does not implement another memory or compiler.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

use candle_core::Device;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uor_r4_core::answer_oracle::FrozenAnswers;
use uor_r4_core::native_geometric::learner::realtext_support::sha256_hex;
use uor_r4_tokenizer::ByteBpeTokenizer;

use crate::stack_checkpoint::{sealed_manifest_sha256, IDENTITY_FILE};
use crate::stack_grounded_session::{
    CompiledAction, CompilerIdentity, GroundedSession, MemoryEffect, SessionScope, SourceSpan,
    TurnControls, TurnOutcome, CHECKPOINT_DIRECTORY, COMPILER_FILE, TOKENIZER_FILE,
};
use crate::stack_store::{HistoryView, StoreRead};
use crate::temporal_compiler::GroundedCompiler;

pub const SCRIPT_SCHEMA: &str = "uor-r4.grounded-script/1";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvalSettings {
    pub max_cases: usize,
    pub max_events: usize,
    pub max_bytes: usize,
    pub max_seconds: u64,
}

impl Default for EvalSettings {
    fn default() -> Self {
        Self {
            max_cases: 64,
            max_events: 512,
            max_bytes: 1 << 20,
            max_seconds: 600,
        }
    }
}

impl EvalSettings {
    pub fn validate(&self) -> Result<(), EvalError> {
        if self.max_cases == 0
            || self.max_events == 0
            || self.max_bytes == 0
            || self.max_seconds == 0
        {
            return Err(EvalError::Input(
                "evaluation limits must be positive".into(),
            ));
        }
        self.max_events
            .checked_mul(2)
            .ok_or_else(|| EvalError::Input("event limit overflow".into()))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptSet {
    pub schema: String,
    pub cases: Vec<ScriptCase>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptCase {
    pub id: String,
    pub source_group: String,
    pub condition: String,
    pub scope: TextScope,
    pub baseline_commit: u64,
    pub baseline_records: usize,
    pub events: Vec<ScriptEvent>,
    pub reload_after: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextScope {
    pub scope: String,
    pub entity: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ScriptEvent {
    StartConversation {
        scope: TextScope,
    },
    User {
        id: String,
        text: String,
        #[serde(default)]
        controls: TurnControls,
        expected: ExpectedTurn,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedTurn {
    pub action: ExpectedAction,
    pub memory: Option<ExpectedMemory>,
    pub answers: Option<FrozenAnswers>,
    pub intervention_answers: Option<FrozenAnswers>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExpectedAction {
    Assert { relation: String, span: SourceSpan },
    Correct { relation: String, span: SourceSpan },
    Query { relation: String, view: HistoryView },
    Unresolved,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExpectedMemory {
    Write {
        value: String,
        id: u64,
        commit: u64,
        conflict: bool,
    },
    WriteDisabled {
        value: String,
    },
    Read {
        status: ExpectedRead,
    },
    Unresolved,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExpectedRead {
    Found {
        value: String,
        record: u64,
        commit: u64,
        conflict: bool,
    },
    Absent,
    NoHistory,
    Evicted,
}

#[derive(Debug)]
pub enum EvalError {
    Input(String),
    Precondition(String),
    Runtime(String),
    Budget { seconds: u64 },
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(s) => write!(f, "script input: {s}"),
            Self::Precondition(s) => write!(f, "script baseline precondition: {s}"),
            Self::Runtime(s) => write!(f, "script runtime: {s}"),
            Self::Budget { seconds } => {
                write!(f, "script cooperative wall limit reached ({seconds}s)")
            }
            Self::Io(e) => write!(f, "script I/O: {e}"),
            Self::Json(e) => write!(f, "script JSON: {e}"),
        }
    }
}
impl std::error::Error for EvalError {}
impl From<std::io::Error> for EvalError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<serde_json::Error> for EvalError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

fn runtime(e: impl fmt::Display) -> EvalError {
    EvalError::Runtime(e.to_string())
}

impl TextScope {
    fn validate(&self) -> Result<(), EvalError> {
        if self.scope.is_empty() || self.entity.trim().is_empty() {
            return Err(EvalError::Input(
                "scope and entity must be explicit and nonempty".into(),
            ));
        }
        Ok(())
    }
    fn encoded(&self, tokenizer: &ByteBpeTokenizer) -> SessionScope {
        SessionScope {
            scope: self.scope.as_bytes().to_vec(),
            entity: tokenizer.encode(&self.entity),
        }
    }
}

impl ScriptSet {
    /// Structural and byte-budget validation, before loading any model.
    pub fn validate(&self, settings: &EvalSettings) -> Result<(), EvalError> {
        settings.validate()?;
        if self.schema != SCRIPT_SCHEMA
            || self.cases.is_empty()
            || self.cases.len() > settings.max_cases
        {
            return Err(EvalError::Input(
                "unsupported schema or case count outside limits".into(),
            ));
        }
        if serde_json::to_vec(self)?.len() > settings.max_bytes {
            return Err(EvalError::Input("script exceeds max_bytes".into()));
        }
        let mut ids = BTreeSet::new();
        let mut events = 0usize;
        for case in &self.cases {
            if case.id.trim().is_empty()
                || !ids.insert(&case.id)
                || case.source_group.trim().is_empty()
                || case.condition.trim().is_empty()
            {
                return Err(EvalError::Input(
                    "case IDs must be unique; IDs/groups/conditions must be nonempty".into(),
                ));
            }
            case.scope.validate()?;
            events = events
                .checked_add(case.events.len())
                .ok_or_else(|| EvalError::Input("event count overflow".into()))?;
            if case.events.is_empty() || events > settings.max_events {
                return Err(EvalError::Input("event count outside limits".into()));
            }
            let mut users = BTreeSet::new();
            let mut answered = false;
            for event in &case.events {
                match event {
                    ScriptEvent::StartConversation { scope } => scope.validate()?,
                    ScriptEvent::User {
                        id,
                        text,
                        controls,
                        expected,
                    } => {
                        if id.trim().is_empty() || !users.insert(id) || text.trim().is_empty() {
                            return Err(EvalError::Input(
                                "user IDs must be unique per case; IDs/text must be nonempty"
                                    .into(),
                            ));
                        }
                        expected.validate(text, *controls)?;
                        answered |= expected.answers.is_some();
                    }
                }
            }
            if !answered {
                return Err(EvalError::Input(
                    "every case needs an original complete-answer expectation".into(),
                ));
            }
            let mut cuts = BTreeSet::new();
            for id in &case.reload_after {
                if !users.contains(id) || !cuts.insert(id) {
                    return Err(EvalError::Input(
                        "reload cuts must name unique user IDs in their case".into(),
                    ));
                }
            }
        }
        Ok(())
    }
}

impl ExpectedAction {
    fn relation(&self) -> Option<&str> {
        match self {
            Self::Assert { relation, .. }
            | Self::Correct { relation, .. }
            | Self::Query { relation, .. } => Some(relation),
            Self::Unresolved => None,
        }
    }
}

impl ExpectedTurn {
    fn validate(&self, text: &str, controls: TurnControls) -> Result<(), EvalError> {
        if self.action.relation().is_some_and(|r| r.trim().is_empty()) {
            return Err(EvalError::Input("empty expected relation".into()));
        }
        let span_value = match self.action {
            ExpectedAction::Assert { span, .. } | ExpectedAction::Correct { span, .. } => Some(
                text.get(span.start..span.end)
                    .filter(|v| !v.trim().is_empty())
                    .ok_or_else(|| {
                        EvalError::Input(
                            "expected span is not a nonempty exact UTF-8 source range".into(),
                        )
                    })?,
            ),
            _ => None,
        };
        if let Some(memory) = &self.memory {
            let valid = match (memory, &self.action) {
                (
                    ExpectedMemory::Write {
                        value, id, commit, ..
                    },
                    ExpectedAction::Assert { .. } | ExpectedAction::Correct { .. },
                ) => controls.write && *id > 0 && *commit > 0 && span_value == Some(value.as_str()),
                (
                    ExpectedMemory::WriteDisabled { value },
                    ExpectedAction::Assert { .. } | ExpectedAction::Correct { .. },
                ) => !controls.write && span_value == Some(value.as_str()),
                (ExpectedMemory::Read { status }, ExpectedAction::Query { .. }) => match status {
                    ExpectedRead::Found {
                        value,
                        record,
                        commit,
                        ..
                    } => !value.is_empty() && *record > 0 && *commit > 0,
                    _ => true,
                },
                (ExpectedMemory::Unresolved, ExpectedAction::Unresolved) => true,
                _ => false,
            };
            if !valid {
                return Err(EvalError::Input(
                    "expected memory disagrees with source/action/controls".into(),
                ));
            }
        }
        for forms in [&self.answers, &self.intervention_answers]
            .into_iter()
            .flatten()
        {
            forms
                .validate()
                .map_err(|e| EvalError::Input(e.to_string()))?;
        }
        if self.intervention_answers.is_some() && self.answers.is_none() {
            return Err(EvalError::Input(
                "intervention answers require an original answer set".into(),
            ));
        }
        Ok(())
    }
}

/// Evaluate with the actual loaded compiler/store/emitter. The caller has
/// exclusively claimed the output root; reload snapshots and per-case receipts
/// are created there without overwrite. The deadline is checked between bounded
/// session operations; it does not interrupt an in-flight model step.
pub fn evaluate(
    base_root: &Path,
    script: &ScriptSet,
    out: &Path,
    settings: &EvalSettings,
) -> Result<Value, EvalError> {
    script.validate(settings)?;
    evaluate_validated(base_root, script, out, settings)
}

type Session = GroundedSession<GroundedCompiler>;

fn open(root: &Path) -> Result<Session, EvalError> {
    let compiler =
        GroundedCompiler::from_bytes(fs::read(root.join(COMPILER_FILE))?).map_err(runtime)?;
    GroundedSession::load(root, compiler, &Device::Cpu).map_err(runtime)
}

fn deadline(start: Instant, settings: &EvalSettings) -> Result<(), EvalError> {
    if start.elapsed().as_secs() >= settings.max_seconds {
        Err(EvalError::Budget {
            seconds: settings.max_seconds,
        })
    } else {
        Ok(())
    }
}

fn envelope_bytes(root: &Path) -> Result<u64, EvalError> {
    let mut total = 0u64;
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let bytes = if kind.is_dir() {
            envelope_bytes(&entry.path())?
        } else if kind.is_file() {
            entry.metadata()?.len()
        } else {
            return Err(EvalError::Precondition(
                "unexpected baseline file kind".into(),
            ));
        };
        total = total
            .checked_add(bytes)
            .ok_or_else(|| EvalError::Input("baseline size overflow".into()))?;
    }
    Ok(total)
}

#[derive(Default, Serialize)]
struct Count {
    pass: usize,
    of: usize,
}
impl Count {
    fn add(&mut self, pass: bool) {
        self.of += 1;
        self.pass += usize::from(pass);
    }
}

#[derive(Default, Serialize)]
struct Scores {
    action: Count,
    memory: Count,
    frozen_complete_answer_membership: Count,
    intervention_complete_answer_membership: Count,
    joint: Count,
    intervention_joint: Count,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RowScore {
    action: bool,
    memory: Option<bool>,
    frozen_complete_answer_membership: Option<bool>,
    intervention_complete_answer_membership: Option<bool>,
    joint: bool,
    intervention_joint: Option<bool>,
}

impl Scores {
    fn add(&mut self, row: &RowScore) {
        self.action.add(row.action);
        if let Some(pass) = row.memory {
            self.memory.add(pass);
        }
        if let Some(pass) = row.frozen_complete_answer_membership {
            self.frozen_complete_answer_membership.add(pass);
        }
        if let Some(pass) = row.intervention_complete_answer_membership {
            self.intervention_complete_answer_membership.add(pass);
        }
        self.joint.add(row.joint);
        if let Some(pass) = row.intervention_joint {
            self.intervention_joint.add(pass);
        }
    }
}

fn action_matches(
    expected: &ExpectedAction,
    actual: &CompiledAction,
    identity: &CompilerIdentity,
) -> bool {
    let relation_matches = |name: &str, id: u32| {
        identity
            .relations
            .iter()
            .any(|r| r.name == name && r.id == id)
    };
    match (expected, actual) {
        (
            ExpectedAction::Assert { relation, span },
            CompiledAction::Assert {
                relation: id,
                span: got,
            },
        )
        | (
            ExpectedAction::Correct { relation, span },
            CompiledAction::Correct {
                relation: id,
                span: got,
            },
        ) => span == got && relation_matches(relation, *id),
        (
            ExpectedAction::Query { relation, view },
            CompiledAction::QueryCurrent { relation: id },
        ) => *view == HistoryView::Current && relation_matches(relation, *id),
        (
            ExpectedAction::Query { relation, view },
            CompiledAction::Query {
                relation: id,
                view: got,
            },
        ) => view == got && relation_matches(relation, *id),
        (ExpectedAction::Unresolved, CompiledAction::Unresolved { .. }) => true,
        _ => false,
    }
}

fn memory_matches(
    expected: &ExpectedMemory,
    actual: &MemoryEffect,
    tokenizer: &ByteBpeTokenizer,
) -> bool {
    let value_matches =
        |value: &str, tokens: &[u32]| tokenizer.decode_bytes(tokens) == value.as_bytes();
    match (expected, actual) {
        (
            ExpectedMemory::Write {
                value,
                id,
                commit,
                conflict,
            },
            MemoryEffect::Write {
                written,
                value_tokens,
            },
        ) => {
            *id == written.id
                && *commit == written.commit
                && *conflict == written.conflict
                && value_matches(value, value_tokens)
        }
        (ExpectedMemory::WriteDisabled { value }, MemoryEffect::WriteDisabled { value_tokens }) => {
            value_matches(value, value_tokens)
        }
        (
            ExpectedMemory::Read {
                status:
                    ExpectedRead::Found {
                        value,
                        record,
                        commit,
                        conflict,
                    },
            },
            MemoryEffect::Read {
                read: StoreRead::Found(found),
            },
        ) => {
            *record == found.record
                && *commit == found.commit
                && *conflict == found.conflict
                && value_matches(value, &found.tokens)
        }
        (
            ExpectedMemory::Read {
                status: ExpectedRead::Absent,
            },
            MemoryEffect::Read {
                read: StoreRead::Absent,
            },
        )
        | (
            ExpectedMemory::Read {
                status: ExpectedRead::NoHistory,
            },
            MemoryEffect::Read {
                read: StoreRead::NoHistory,
            },
        )
        | (
            ExpectedMemory::Read {
                status: ExpectedRead::Evicted,
            },
            MemoryEffect::Read {
                read: StoreRead::Evicted,
            },
        )
        | (ExpectedMemory::Unresolved, MemoryEffect::Unresolved) => true,
        _ => false,
    }
}

fn score(
    expected: &ExpectedTurn,
    outcome: Option<&TurnOutcome>,
    identity: &CompilerIdentity,
    tokenizer: &ByteBpeTokenizer,
) -> RowScore {
    let action =
        outcome.is_some_and(|outcome| action_matches(&expected.action, &outcome.action, identity));
    let memory = expected.memory.as_ref().map(|expected| {
        outcome.is_some_and(|outcome| memory_matches(expected, &outcome.memory, tokenizer))
    });
    let original = expected
        .answers
        .as_ref()
        .map(|forms| outcome.is_some_and(|outcome| forms.accepts(&outcome.reply_text)));
    let intervention = expected
        .intervention_answers
        .as_ref()
        .map(|forms| outcome.is_some_and(|outcome| forms.accepts(&outcome.reply_text)));
    let mechanism = action && memory.unwrap_or(true);
    RowScore {
        action,
        memory,
        frozen_complete_answer_membership: original,
        intervention_complete_answer_membership: intervention,
        joint: mechanism && original.unwrap_or(true),
        intervention_joint: intervention.map(|pass| mechanism && pass),
    }
}

fn expected_status(expected: &ExpectedTurn) -> &'static str {
    match &expected.memory {
        Some(ExpectedMemory::Write { conflict: true, .. }) => "write_conflict",
        Some(ExpectedMemory::Write { .. }) => "write",
        Some(ExpectedMemory::WriteDisabled { .. }) => "write_disabled",
        Some(ExpectedMemory::Read {
            status: ExpectedRead::Found { conflict: true, .. },
        }) => "read_found_conflict",
        Some(ExpectedMemory::Read {
            status: ExpectedRead::Found { .. },
        }) => "read_found",
        Some(ExpectedMemory::Read {
            status: ExpectedRead::Absent,
        }) => "read_absent",
        Some(ExpectedMemory::Read {
            status: ExpectedRead::NoHistory,
        }) => "read_no_history",
        Some(ExpectedMemory::Read {
            status: ExpectedRead::Evicted,
        }) => "read_evicted",
        Some(ExpectedMemory::Unresolved) => "unresolved",
        None => "unannotated",
    }
}

fn actual_status(outcome: Option<&TurnOutcome>) -> &'static str {
    match outcome.map(|o| &o.memory) {
        Some(MemoryEffect::Write { written, .. }) if written.conflict => "write_conflict",
        Some(MemoryEffect::Write { .. }) => "write",
        Some(MemoryEffect::WriteDisabled { .. }) => "write_disabled",
        Some(MemoryEffect::Read {
            read: StoreRead::Found(v),
        }) if v.conflict => "read_found_conflict",
        Some(MemoryEffect::Read {
            read: StoreRead::Found(_),
        }) => "read_found",
        Some(MemoryEffect::Read {
            read: StoreRead::Absent,
        }) => "read_absent",
        Some(MemoryEffect::Read {
            read: StoreRead::NoHistory,
        }) => "read_no_history",
        Some(MemoryEffect::Read {
            read: StoreRead::Evicted,
        }) => "read_evicted",
        Some(MemoryEffect::Unresolved) => "unresolved",
        None => "error",
    }
}

fn token_hash(ids: &[u32]) -> String {
    sha256_hex(
        &ids.iter()
            .flat_map(|id| id.to_le_bytes())
            .collect::<Vec<_>>(),
    )
}

struct Pass {
    rows: Vec<Value>,
    outcomes: Vec<Option<TurnOutcome>>,
    scores: Scores,
    expected_cohorts: BTreeMap<&'static str, Scores>,
    actual_cohorts: BTreeMap<&'static str, Scores>,
    cuts: Vec<Value>,
    scope: SessionScope,
    history: Vec<u32>,
    store: Vec<u8>,
}

fn observe(file: &mut fs::File, value: &Value) -> Result<(), EvalError> {
    file.write_all(&serde_json::to_vec(value)?)?;
    file.write_all(b"\n")?;
    Ok(())
}

fn baseline(root: &Path, digest: &str, case: &ScriptCase) -> Result<Session, EvalError> {
    if sealed_manifest_sha256(root).map_err(runtime)? != digest {
        return Err(EvalError::Precondition(
            "baseline envelope changed during evaluation".into(),
        ));
    }
    let session = open(root)?;
    if session.store().commit() != case.baseline_commit
        || session.store().records() != case.baseline_records
    {
        return Err(EvalError::Precondition(format!(
            "{} expected commit {} and {} records, loaded commit {} and {} records",
            case.id,
            case.baseline_commit,
            case.baseline_records,
            session.store().commit(),
            session.store().records()
        )));
    }
    Ok(session)
}

fn run_case(
    mut session: Session,
    case: &ScriptCase,
    case_index: usize,
    reload: bool,
    tokenizer: &ByteBpeTokenizer,
    out: &Path,
    observations: &mut fs::File,
    started: Instant,
    settings: &EvalSettings,
) -> Result<Pass, EvalError> {
    session
        .start_conversation(case.scope.encoded(tokenizer))
        .map_err(runtime)?;
    let mut pass = Pass {
        rows: Vec::new(),
        outcomes: Vec::new(),
        scores: Scores::default(),
        expected_cohorts: BTreeMap::new(),
        actual_cohorts: BTreeMap::new(),
        cuts: Vec::new(),
        scope: session.scope().clone(),
        history: Vec::new(),
        store: Vec::new(),
    };
    for (event_index, event) in case.events.iter().enumerate() {
        deadline(started, settings)?;
        match event {
            ScriptEvent::StartConversation { scope } => {
                session
                    .start_conversation(scope.encoded(tokenizer))
                    .map_err(runtime)?;
                observe(
                    observations,
                    &json!({"case":case.id,"pass":if reload {"reloaded"}else{"uninterrupted"},"event":event_index,"start_conversation":scope,"store_commit":session.store().commit()}),
                )?;
            }
            ScriptEvent::User {
                id,
                text,
                controls,
                expected,
            } => {
                let result = session.turn_with_controls(text, *controls);
                let (outcome, error) = match result {
                    Ok(outcome) => (Some(outcome), None),
                    Err(error) => (None, Some(error.to_string())),
                };
                let scored = score(
                    expected,
                    outcome.as_ref(),
                    session.compiler_identity(),
                    tokenizer,
                );
                pass.scores.add(&scored);
                pass.expected_cohorts
                    .entry(expected_status(expected))
                    .or_default()
                    .add(&scored);
                pass.actual_cohorts
                    .entry(actual_status(outcome.as_ref()))
                    .or_default()
                    .add(&scored);
                let row = json!({
                    "id":id,"event":event_index,"text":text,"controls":controls,"expected":expected,
                    "scores":scored,"outcome":outcome,"error":error,
                    "input":{"source_bytes":text.len(),"source_sha256":sha256_hex(text.as_bytes()),
                        "emitter_tokens":outcome.as_ref().map(|o| o.emitter_input_ids.len()),
                        "emitter_u32le_sha256":outcome.as_ref().map(|o| token_hash(&o.emitter_input_ids))},
                });
                observe(
                    observations,
                    &json!({"case":case.id,"pass":if reload {"reloaded"}else{"uninterrupted"},"row":row}),
                )?;
                pass.rows.push(row);
                pass.outcomes.push(outcome);
                deadline(started, settings)?;
                if reload && case.reload_after.contains(id) {
                    let snapshot =
                        out.join(format!("reload-{case_index:04}-{:04}", pass.cuts.len()));
                    let before_store = session.store().to_bytes().map_err(runtime)?;
                    let before_history = session.history_ids().to_vec();
                    let before_turns = session.turns().to_vec();
                    let before_scope = session.scope().clone();
                    let before_compiler = session.compiler_identity().clone();
                    session.save(&snapshot).map_err(runtime)?;
                    drop(session);
                    session = open(&snapshot)?;
                    let equal = session.store().to_bytes().map_err(runtime)? == before_store
                        && session.history_ids() == before_history
                        && session.turns() == before_turns
                        && session.scope() == &before_scope
                        && session.compiler_identity() == &before_compiler;
                    let cut = json!({"after_user":id,"snapshot":snapshot.file_name().and_then(|s|s.to_str()),"restored_state_equal":equal});
                    observe(observations, &json!({"case":case.id,"reload_cut":cut}))?;
                    pass.cuts.push(cut);
                    deadline(started, settings)?;
                }
            }
        }
    }
    pass.scope = session.scope().clone();
    pass.history = session.history_ids().to_vec();
    pass.store = session.store().to_bytes().map_err(runtime)?;
    Ok(pass)
}

fn continuity(first: &Pass, second: &Pass) -> Value {
    let mut compared = Count::default();
    let mut failed_pairs = 0usize;
    for (a, b) in first.outcomes.iter().zip(&second.outcomes) {
        match (a, b) {
            (Some(a), Some(b)) => compared.add(a == b),
            _ => {
                compared.add(false);
                failed_pairs += 1;
            }
        }
    }
    let state_equal = first.scope == second.scope
        && first.history == second.history
        && first.store == second.store;
    let all_cuts_equal = second
        .cuts
        .iter()
        .all(|c| c["restored_state_equal"] == true);
    json!({
        "rule":"same ordered script, caller controls and exact compiler bytes; disk reload in this process",
        "outcomes":compared,"failed_pairs":failed_pairs,
        "final_scope_history_store_equal":state_equal,"cuts":second.cuts,
        "pass":!first.outcomes.is_empty() && first.outcomes.len()==second.outcomes.len()
            && compared.pass==first.outcomes.len() && state_equal && all_cuts_equal,
    })
}

fn evaluate_validated(
    base_root: &Path,
    script: &ScriptSet,
    out: &Path,
    settings: &EvalSettings,
) -> Result<Value, EvalError> {
    let started = Instant::now();
    let manifest = sealed_manifest_sha256(base_root).map_err(runtime)?;
    let baseline_bytes = envelope_bytes(base_root)?;
    let cuts = script
        .cases
        .iter()
        .map(|case| case.reload_after.len())
        .sum::<usize>();
    let baseline_sized_copies = baseline_bytes
        .checked_mul(
            u64::try_from(cuts).map_err(|_| EvalError::Input("reload count overflow".into()))?,
        )
        .ok_or_else(|| EvalError::Input("reload copy projection overflow".into()))?;
    let mut probe = open(base_root)?;
    let tokenizer_bytes = fs::read(base_root.join(TOKENIZER_FILE))?;
    if sha256_hex(&tokenizer_bytes) != probe.compiler_identity().tokenizer_sha256 {
        return Err(EvalError::Precondition(
            "baseline tokenizer changed after session load".into(),
        ));
    }
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| EvalError::Input("unreadable baseline tokenizer".into()))?;
    // Check every baseline, label and caller address before any inference.
    for case in &script.cases {
        if probe.store().commit() != case.baseline_commit
            || probe.store().records() != case.baseline_records
        {
            return Err(EvalError::Precondition(format!(
                "{} baseline commit/record count differs",
                case.id
            )));
        }
        probe
            .start_conversation(case.scope.encoded(&tokenizer))
            .map_err(|e| EvalError::Input(e.to_string()))?;
        for event in &case.events {
            match event {
                ScriptEvent::StartConversation { scope } => probe
                    .start_conversation(scope.encoded(&tokenizer))
                    .map_err(|e| EvalError::Input(e.to_string()))?,
                ScriptEvent::User { expected, .. } => {
                    if let Some(name) = expected.action.relation() {
                        if !probe
                            .compiler_identity()
                            .relations
                            .iter()
                            .any(|r| r.name == name)
                        {
                            return Err(EvalError::Input(format!(
                                "expected relation {name} is outside loaded compiler schema"
                            )));
                        }
                    }
                }
            }
        }
    }
    let checkpoint_root = base_root.join(CHECKPOINT_DIRECTORY);
    let binding = json!({
        "session_manifest_sha256":manifest,
        "checkpoint_manifest_sha256":sealed_manifest_sha256(&checkpoint_root).map_err(runtime)?,
        "checkpoint_record":serde_json::from_slice::<Value>(&fs::read(checkpoint_root.join(IDENTITY_FILE))?)?,
        "compiler":probe.compiler_identity(),
        "tokenizer_sha256":sha256_hex(&tokenizer_bytes),
        "store":{"lineage":probe.store().lineage(),"capacity":probe.store().capacity(),"commit":probe.store().commit(),"records":probe.store().records()},
        "session_limits":probe.limits(),
    });
    drop(probe);
    deadline(started, settings)?;
    let mut observations = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out.join("observations.jsonl"))?;
    let mut reports = Vec::new();
    let mut total = Scores::default();
    for (index, case) in script.cases.iter().enumerate() {
        deadline(started, settings)?;
        let first = run_case(
            baseline(base_root, &manifest, case)?,
            case,
            index,
            false,
            &tokenizer,
            out,
            &mut observations,
            started,
            settings,
        )?;
        for row in &first.rows {
            let scored: RowScore = serde_json::from_value(row["scores"].clone())?;
            total.add(&scored);
        }
        let reloaded = if case.reload_after.is_empty() {
            None
        } else {
            deadline(started, settings)?;
            let second = run_case(
                baseline(base_root, &manifest, case)?,
                case,
                index,
                true,
                &tokenizer,
                out,
                &mut observations,
                started,
                settings,
            )?;
            Some(continuity(&first, &second))
        };
        let report = json!({
            "id":case.id,"source_group":case.source_group,"condition":case.condition,
            "condition_status":"caller-supplied metadata, not inferred or qualified",
            "scores":first.scores,"expected_memory_cohorts":first.expected_cohorts,
            "actual_memory_cohorts":first.actual_cohorts,"rows":first.rows,"reload":reloaded,
            "final_state":{"scope":first.scope,"history_tokens":first.history.len(),"history_u32le_sha256":token_hash(&first.history),"store_bytes":first.store.len(),"store_sha256":sha256_hex(&first.store)},
        });
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(out.join(format!("case-{index:04}.json")))?;
        file.write_all(&serde_json::to_vec_pretty(&report)?)?;
        reports.push(report);
        deadline(started, settings)?;
    }
    Ok(json!({
        "schema":"uor-r4.grounded-script-evaluation/1","script_schema":script.schema,
        "script_canonical_sha256":sha256_hex(&serde_json::to_vec(script)?),
        "baseline":binding,"settings":settings,"scores":total,"cases":reports,
        "history":"generated, with explicit caller conversation restarts only",
        "scoring":"frozen_complete_answer_membership: exact bytes in supplied original answer sets; unlisted legitimate paraphrases can fail; no semantic-family claim",
        "joint_rule":"every user row, including errors; action AND each supplied memory/original-answer expectation; optional setup answers have no answer denominator",
        "intervention_rule":"original answers remain fixed; intervention answers and their joint score are separately reported",
        "training_exposure":"UNVERIFIED; source_group is caller metadata",
        "budget_scope":"max_events bounds original script events; cases with cuts have one additional pass; deadline is cooperative between session operations, not a hard per-step timer",
        "reload_storage":{"cuts":cuts,"baseline_envelope_bytes":baseline_bytes,"baseline_sized_copies_bytes":baseline_sized_copies,
            "scope":"copy-size estimate only, not a storage cap; growing store/history, observations and reports add bytes; actual-artifact execution requires separate storage admission"},
        "planned_event_executions":script.cases.iter().map(|c|c.events.len() * if c.reload_after.is_empty(){1}else{2}).sum::<usize>(),
        "wall_seconds":started.elapsed().as_secs_f64(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stack_grounded_session::{RecallDisposition, RelationLabel, TurnStop};
    use crate::stack_store::{StoreValue, Update};
    use uor_r4_core::answer_oracle::RecordedValueIntent;

    fn script() -> ScriptSet {
        serde_json::from_value(json!({
            "schema":SCRIPT_SCHEMA,
            "cases":[{
                "id":"temporal","source_group":"TEST_ONLY/authored","condition":"construction",
                "scope":{"scope":"project-a","entity":"user"},
                "baseline_commit":0,"baseline_records":0,
                "events":[{
                    "kind":"user","id":"q","text":"What was the initial value?",
                    "expected":{
                        "action":{"kind":"query","relation":"value","view":"Initial"},
                        "memory":{"kind":"read","status":{"kind":"found","value":"a","record":1,"commit":1,"conflict":false}},
                        "answers":{"intent":"initial","accepted":["a."]}
                    }
                }],
                "reload_after":["q"]
            }]
        })).expect("typed script")
    }

    fn tokenizer() -> ByteBpeTokenizer {
        ByteBpeTokenizer::from_tokenizer_json_bytes(br#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"a":0,"b":1},"merges":[]}}"#).expect("tiny tokenizer")
    }

    fn identity() -> CompilerIdentity {
        CompilerIdentity {
            schema: "TEST_ONLY".into(),
            artifact_sha256: "0".repeat(64),
            tokenizer_sha256: "1".repeat(64),
            label_schema: "TEST_ONLY".into(),
            relations: vec![RelationLabel {
                id: 17,
                name: "value".into(),
            }],
            encoder: None,
        }
    }

    fn outcome() -> TurnOutcome {
        TurnOutcome {
            source: "What was the initial value?".into(),
            action: CompiledAction::Query {
                relation: 17,
                view: HistoryView::Initial,
            },
            controls: TurnControls::default(),
            memory: MemoryEffect::Read {
                read: StoreRead::Found(StoreValue {
                    tokens: vec![0],
                    record: 1,
                    commit: 1,
                    update: Update::Assert,
                    conflict: false,
                }),
            },
            recall: RecallDisposition::Value,
            memory_commit: 1,
            emitter_input_ids: vec![0, 1],
            retained_from_turn: 0,
            reply_ids: vec![0],
            reply_text: "a.".into(),
            stop: TurnStop::MaxNewTokens,
            caller_eos_inserted: true,
        }
    }

    fn expected() -> ExpectedTurn {
        let mut cases = script().cases;
        match cases.remove(0).events.remove(0) {
            ScriptEvent::User { expected, .. } => expected,
            _ => panic!("user fixture"),
        }
    }

    #[test]
    fn typed_scripts_reject_ambiguous_invalid_and_over_budget_input_before_model_load() {
        let original = script();
        original.validate(&EvalSettings::default()).expect("valid");
        let encoded = serde_json::to_value(&original).expect("json");
        for (pointer, value) in [
            ("/schema", json!("unknown")),
            ("/cases/0/events/0/id", json!("")),
            ("/cases/0/scope/entity", json!("")),
            ("/cases/0/events/0/expected/answers/accepted", json!([])),
            ("/cases/0/reload_after", json!(["q", "q"])),
            ("/cases/0/reload_after", json!(["missing"])),
        ] {
            let mut bad = encoded.clone();
            *bad.pointer_mut(pointer).expect("pointer") = value;
            let bad: ScriptSet = serde_json::from_value(bad).expect("shape valid");
            assert!(matches!(
                bad.validate(&EvalSettings::default()),
                Err(EvalError::Input(_))
            ));
        }
        let mut duplicate = original.clone();
        duplicate.cases.push(duplicate.cases[0].clone());
        assert!(duplicate.validate(&EvalSettings::default()).is_err());
        let mut wrong_field = encoded.clone();
        wrong_field["cases"][0]["events"][0]["hidden_oracle"] = "a".into();
        assert!(serde_json::from_value::<ScriptSet>(wrong_field).is_err());
        let mut settings = EvalSettings::default();
        settings.max_events = 0;
        assert!(settings.validate().is_err());
        settings = EvalSettings::default();
        settings.max_bytes = 1;
        assert!(original.validate(&settings).is_err());
        // Malformed expected byte spans are refused before any session opens.
        let mut write = expected();
        write.action = ExpectedAction::Correct {
            relation: "value".into(),
            span: SourceSpan { start: 1, end: 2 },
        };
        write.memory = Some(ExpectedMemory::WriteDisabled { value: "é".into() });
        assert!(write
            .validate(
                "é",
                TurnControls {
                    read: true,
                    write: false
                }
            )
            .is_err());
        write.action = ExpectedAction::Correct {
            relation: "value".into(),
            span: SourceSpan { start: 0, end: 2 },
        };
        write
            .validate(
                "é",
                TurnControls {
                    read: true,
                    write: false,
                },
            )
            .expect("exact source");
        assert!(write.validate("é", TurnControls::default()).is_err());
    }

    #[test]
    fn scoring_preserves_views_errors_denominators_and_original_intervention_answers() {
        let tokenizer = tokenizer();
        let mut expected = expected();
        let mut actual = outcome();
        assert!(score(&expected, Some(&actual), &identity(), &tokenizer).joint);
        actual.action = CompiledAction::QueryCurrent { relation: 17 };
        assert!(!score(&expected, Some(&actual), &identity(), &tokenizer).action);
        expected.action = ExpectedAction::Query {
            relation: "value".into(),
            view: HistoryView::Current,
        };
        assert!(score(&expected, Some(&actual), &identity(), &tokenizer).action);
        actual.action = CompiledAction::Query {
            relation: 17,
            view: HistoryView::Current,
        };
        assert!(score(&expected, Some(&actual), &identity(), &tokenizer).action);
        expected.intervention_answers = Some(FrozenAnswers {
            intent: RecordedValueIntent::Current,
            accepted: vec!["b.".into()],
        });
        actual.reply_text = "b.".into();
        let intervention = score(&expected, Some(&actual), &identity(), &tokenizer);
        assert_eq!(intervention.frozen_complete_answer_membership, Some(false));
        assert_eq!(
            intervention.intervention_complete_answer_membership,
            Some(true)
        );
        assert!(!intervention.joint);
        assert_eq!(intervention.intervention_joint, Some(true));
        actual.reply_text = "a. ".into();
        assert_eq!(
            score(&expected, Some(&actual), &identity(), &tokenizer)
                .frozen_complete_answer_membership,
            Some(false)
        );
        let failed = score(&expected, None, &identity(), &tokenizer);
        assert!(!failed.joint);
        assert_eq!(failed.memory, Some(false));
        assert_eq!(failed.frozen_complete_answer_membership, Some(false));
        let mut scores = Scores::default();
        scores.add(&failed);
        expected.answers = None;
        expected.intervention_answers = None;
        let setup = score(&expected, Some(&actual), &identity(), &tokenizer);
        scores.add(&setup);
        assert_eq!(
            (
                scores.action.of,
                scores.joint.of,
                scores.frozen_complete_answer_membership.of
            ),
            (2, 2, 1)
        );
        assert_eq!(scores.frozen_complete_answer_membership.pass, 0);
    }

    #[test]
    fn read_disable_does_not_change_typed_memory_and_statuses_do_not_collapse() {
        let tokenizer = tokenizer();
        let expected = expected();
        let mut actual = outcome();
        actual.controls.read = false;
        actual.recall = RecallDisposition::Disabled;
        assert_eq!(
            score(&expected, Some(&actual), &identity(), &tokenizer).memory,
            Some(true)
        );
        let wanted = [
            ExpectedRead::Absent,
            ExpectedRead::NoHistory,
            ExpectedRead::Evicted,
            ExpectedRead::Found {
                value: "a".into(),
                record: 1,
                commit: 1,
                conflict: false,
            },
        ];
        let got = [
            StoreRead::Absent,
            StoreRead::NoHistory,
            StoreRead::Evicted,
            StoreRead::Found(StoreValue {
                tokens: vec![0],
                record: 1,
                commit: 1,
                update: Update::Assert,
                conflict: false,
            }),
        ];
        for (i, wanted) in wanted.into_iter().enumerate() {
            for (j, got) in got.iter().enumerate() {
                assert_eq!(
                    memory_matches(
                        &ExpectedMemory::Read {
                            status: wanted.clone()
                        },
                        &MemoryEffect::Read { read: got.clone() },
                        &tokenizer
                    ),
                    i == j
                );
            }
        }
        actual.memory = MemoryEffect::Read {
            read: StoreRead::Found(StoreValue {
                tokens: vec![0],
                record: 1,
                commit: 1,
                update: Update::Assert,
                conflict: true,
            }),
        };
        assert_eq!(actual_status(Some(&actual)), "read_found_conflict");
        assert_eq!(
            score(&expected, Some(&actual), &identity(), &tokenizer).memory,
            Some(false)
        );
    }

    fn pass(outcomes: Vec<Option<TurnOutcome>>) -> Pass {
        Pass {
            rows: Vec::new(),
            outcomes,
            scores: Scores::default(),
            expected_cohorts: BTreeMap::new(),
            actual_cohorts: BTreeMap::new(),
            cuts: vec![json!({"restored_state_equal":true})],
            scope: SessionScope {
                scope: vec![1],
                entity: vec![2],
            },
            history: vec![3],
            store: vec![4],
        }
    }

    #[test]
    fn reload_equality_requires_successful_outcomes_and_exact_final_state() {
        assert_eq!(
            continuity(&pass(vec![None]), &pass(vec![None]))["pass"],
            false
        );
        let first = pass(vec![Some(outcome())]);
        let mut second = pass(vec![Some(outcome())]);
        assert_eq!(continuity(&first, &second)["pass"], true);
        second.store.push(5);
        assert_eq!(continuity(&first, &second)["pass"], false);
        second = pass(vec![Some(outcome())]);
        second.cuts[0]["restored_state_equal"] = json!(false);
        assert_eq!(continuity(&first, &second)["pass"], false);
    }
}
