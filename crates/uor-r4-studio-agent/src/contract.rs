//! Run-scoped agent event envelope, payloads, cursor and approval contract.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::canonical::{is_digest_string, stable_json};
use crate::error::{AgentError, AgentErrorCode};
use crate::ids::{event_id, is_valid_id};
use crate::{
    AGENT_ERROR_SCHEMA, AGENT_EVENT_SCHEMA, APPROVAL_TTL_MS, CANONICAL_JSON_SCHEMA, CURSOR_PREFIX,
    MAX_EVENTS_PER_RUN, MAX_EVENT_BYTES, MAX_SAFE_INTEGER,
};

pub const KNOWN_EVENT_TYPES: [&str; 30] = [
    "run.queued",
    "run.started",
    "run.status",
    "run.finished",
    "run.failed",
    "run.cancelled",
    "run.expired",
    "run.resumed",
    "turn.started",
    "turn.finished",
    "assistant.delta",
    "assistant.message",
    "thinking.delta",
    "thinking.message",
    "plan.updated",
    "message.queued",
    "message.steered",
    "message.forced",
    "tool.requested",
    "tool.approval_required",
    "tool.approval_resolved",
    "tool.started",
    "tool.completed",
    "tool.failed",
    "artifact.created",
    "changes.snapshot",
    "diff.comment",
    "usage.updated",
    "heartbeat",
    "error",
];

pub fn is_known_event_type(event_type: &str) -> bool {
    KNOWN_EVENT_TYPES.contains(&event_type)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentEvent {
    pub schema: String,
    pub event_id: String,
    pub project_id: String,
    pub task_id: String,
    pub run_id: String,
    pub generation: u64,
    pub sequence: u64,
    pub recorded_at_ms: u64,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: Value,
}

impl AgentEvent {
    pub fn is_authoritative_terminal(&self) -> bool {
        matches!(
            self.event_type.as_str(),
            "run.finished" | "run.failed" | "run.cancelled" | "run.expired"
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
    Queued,
    Running,
    WaitingForApproval,
    WaitingForBackground,
    Cancelling,
    Finished,
    Failed,
    Cancelled,
    Expired,
}

impl RunState {
    pub const ALL: [RunState; 9] = [
        RunState::Queued,
        RunState::Running,
        RunState::WaitingForApproval,
        RunState::WaitingForBackground,
        RunState::Cancelling,
        RunState::Finished,
        RunState::Failed,
        RunState::Cancelled,
        RunState::Expired,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            RunState::Queued => "queued",
            RunState::Running => "running",
            RunState::WaitingForApproval => "waiting_for_approval",
            RunState::WaitingForBackground => "waiting_for_background",
            RunState::Cancelling => "cancelling",
            RunState::Finished => "finished",
            RunState::Failed => "failed",
            RunState::Cancelled => "cancelled",
            RunState::Expired => "expired",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        RunState::ALL
            .iter()
            .copied()
            .find(|state| state.as_str() == value)
    }

    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            RunState::Finished | RunState::Failed | RunState::Cancelled | RunState::Expired
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Archived,
    Queued,
    Running,
    WaitingForApproval,
    WaitingForBackground,
    Cancelling,
    Finished,
    Failed,
    Cancelled,
    Expired,
    Idle,
}

impl TaskState {
    pub fn as_str(self) -> &'static str {
        match self {
            TaskState::Archived => "archived",
            TaskState::Queued => "queued",
            TaskState::Running => "running",
            TaskState::WaitingForApproval => "waiting_for_approval",
            TaskState::WaitingForBackground => "waiting_for_background",
            TaskState::Cancelling => "cancelling",
            TaskState::Finished => "finished",
            TaskState::Failed => "failed",
            TaskState::Cancelled => "cancelled",
            TaskState::Expired => "expired",
            TaskState::Idle => "idle",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveRun {
    pub run_id: String,
    pub state: RunState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskSummary {
    pub task_id: String,
    pub project_id: String,
    pub title: String,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub hold: bool,
    #[serde(default)]
    pub active_run: Option<ActiveRun>,
}

impl TaskSummary {
    pub fn state(&self) -> TaskState {
        if self.archived {
            return TaskState::Archived;
        }
        if let Some(active) = &self.active_run {
            return match active.state {
                RunState::Queued => TaskState::Queued,
                RunState::Running => TaskState::Running,
                RunState::WaitingForApproval => TaskState::WaitingForApproval,
                RunState::WaitingForBackground => TaskState::WaitingForBackground,
                RunState::Cancelling => TaskState::Cancelling,
                RunState::Finished | RunState::Failed | RunState::Cancelled | RunState::Expired => {
                    if self.hold {
                        TaskState::WaitingForBackground
                    } else {
                        match active.state {
                            RunState::Finished => TaskState::Finished,
                            RunState::Failed => TaskState::Failed,
                            RunState::Cancelled => TaskState::Cancelled,
                            RunState::Expired => TaskState::Expired,
                            _ => TaskState::Idle,
                        }
                    }
                }
            };
        }
        if self.hold {
            TaskState::WaitingForBackground
        } else {
            TaskState::Idle
        }
    }
}

pub fn validate_event(value: &Value) -> Result<AgentEvent, AgentError> {
    let event: AgentEvent = serde_json::from_value(value.clone()).map_err(|error| {
        AgentError::new(
            AgentErrorCode::MalformedEvent,
            format!("event envelope is malformed: {error}"),
        )
    })?;

    if event.schema != AGENT_EVENT_SCHEMA {
        return Err(AgentError::invalid_event(format!(
            "event schema is {:?}, expected {AGENT_EVENT_SCHEMA:?}",
            event.schema
        )));
    }
    if !is_valid_id(&event.project_id, "prj") {
        return Err(AgentError::invalid_event(
            "event project_id is not a prj_<16 hex> id",
        ));
    }
    if !is_valid_id(&event.task_id, "tsk") {
        return Err(AgentError::invalid_event(
            "event task_id is not a tsk_<16 hex> id",
        ));
    }
    if !is_valid_id(&event.run_id, "run") {
        return Err(AgentError::invalid_event(
            "event run_id is not a run_<16 hex> id",
        ));
    }
    if event.generation == 0 || event.generation > MAX_SAFE_INTEGER {
        return Err(AgentError::invalid_event(
            "event generation must be a safe integer >= 1",
        ));
    }
    if event.sequence > MAX_SAFE_INTEGER {
        return Err(AgentError::invalid_event(
            "event sequence must be a safe integer >= 0",
        ));
    }
    if event.recorded_at_ms > MAX_SAFE_INTEGER {
        return Err(AgentError::invalid_event(
            "event recorded_at_ms must be a safe non-negative integer",
        ));
    }
    if !is_event_type_name(&event.event_type) {
        return Err(AgentError::invalid_event(format!(
            "event type {:?} is not a lower-case dotted name",
            event.event_type
        )));
    }
    if event.event_id != event_id(&event.run_id, event.generation, event.sequence) {
        return Err(AgentError::invalid_event(
            "event_id does not match run_id, generation and sequence",
        ));
    }
    if !event.data.is_object() {
        return Err(AgentError::invalid_event("event data must be an object"));
    }
    if is_known_event_type(&event.event_type) {
        validate_known_data(&event)?;
    }
    crate::canonical::canonical_json(&event.data)?;
    Ok(event)
}

pub fn validate_run_start_pair(
    queued: &Value,
    started: &Value,
) -> Result<(AgentEvent, AgentEvent), AgentError> {
    let queued = validate_event(queued)?;
    let started = validate_event(started)?;
    if queued.event_type != "run.queued" || queued.sequence != 0 {
        return Err(AgentError::invalid_event(
            "a fresh run must start with run.queued at sequence 0",
        ));
    }
    if started.event_type != "run.started" || started.sequence != 1 {
        return Err(AgentError::invalid_event(
            "a fresh run must contain run.started at sequence 1",
        ));
    }
    if queued.project_id != started.project_id
        || queued.task_id != started.task_id
        || queued.run_id != started.run_id
        || queued.generation != started.generation
    {
        return Err(AgentError::invalid_event(
            "run.queued and run.started must share project, task, run and generation",
        ));
    }
    Ok((queued, started))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub approval_id: String,
    pub task_id: String,
    pub run_id: String,
    pub generation: u64,
    pub tool_call_id: String,
    pub tool: String,
    pub scope: String,
    pub request_digest: String,
    pub requested_at_ms: u64,
    pub expires_at_ms: u64,
}

pub fn validate_approval(value: &Value) -> Result<ApprovalRequest, AgentError> {
    let approval: ApprovalRequest = serde_json::from_value(value.clone()).map_err(|error| {
        AgentError::invalid_event(format!("approval identity is malformed: {error}"))
    })?;
    if !is_valid_id(&approval.approval_id, "apr") {
        return Err(AgentError::invalid_event(
            "approval_id is not an apr_<16 hex> id",
        ));
    }
    if !is_valid_id(&approval.task_id, "tsk") {
        return Err(AgentError::invalid_event(
            "approval task_id is not a tsk_<16 hex> id",
        ));
    }
    if !is_valid_id(&approval.run_id, "run") {
        return Err(AgentError::invalid_event(
            "approval run_id is not a run_<16 hex> id",
        ));
    }
    if approval.generation == 0 || approval.generation > MAX_SAFE_INTEGER {
        return Err(AgentError::invalid_event(
            "approval generation must be a safe integer >= 1",
        ));
    }
    if approval.tool_call_id.is_empty() {
        return Err(AgentError::invalid_event(
            "approval tool_call_id is required",
        ));
    }
    if approval.tool.is_empty() {
        return Err(AgentError::invalid_event("approval tool is required"));
    }
    if approval.scope.is_empty() {
        return Err(AgentError::invalid_event("approval scope is required"));
    }
    if !is_digest_string(&approval.request_digest) {
        return Err(AgentError::invalid_event(
            "approval request_digest must be sha256:<64 lowercase hex>",
        ));
    }
    if approval.requested_at_ms > MAX_SAFE_INTEGER || approval.expires_at_ms > MAX_SAFE_INTEGER {
        return Err(AgentError::invalid_event(
            "approval timestamps must be safe non-negative integers",
        ));
    }
    if approval.expires_at_ms <= approval.requested_at_ms {
        return Err(AgentError::invalid_event(
            "approval expires_at_ms must be after requested_at_ms",
        ));
    }
    Ok(approval)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCursor {
    pub project_id: String,
    pub task_id: String,
    pub run_id: String,
    pub generation: u64,
    pub sequence: u64,
}

impl AgentCursor {
    pub fn from_event(event: &AgentEvent) -> Result<Self, AgentError> {
        if event.generation == 0 {
            return Err(AgentError::invalid_event(
                "a cursor cannot be built from generation 0",
            ));
        }
        Ok(Self {
            project_id: event.project_id.clone(),
            task_id: event.task_id.clone(),
            run_id: event.run_id.clone(),
            generation: event.generation,
            sequence: event.sequence,
        })
    }

    pub fn encode(&self) -> String {
        format!(
            "{CURSOR_PREFIX}.{}.{}.{}.{}.{}",
            b64url_encode(self.project_id.as_bytes()),
            b64url_encode(self.task_id.as_bytes()),
            b64url_encode(self.run_id.as_bytes()),
            self.generation,
            self.sequence
        )
    }

    pub fn parse(token: &str) -> Result<Self, AgentError> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 6 || parts[0] != CURSOR_PREFIX {
            return Err(AgentError::invalid_event(
                "cursor token must be uorse1.<project>.<task>.<run>.<generation>.<sequence>",
            ));
        }
        let project_id = decode_id(parts[1], "prj")?;
        let task_id = decode_id(parts[2], "tsk")?;
        let run_id = decode_id(parts[3], "run")?;
        let generation = parse_decimal(parts[4], "generation")?;
        let sequence = parse_decimal(parts[5], "sequence")?;
        if generation == 0 {
            return Err(AgentError::invalid_event("cursor generation must be >= 1"));
        }
        Ok(Self {
            project_id,
            task_id,
            run_id,
            generation,
            sequence,
        })
    }

    pub fn last_event_id(&self) -> String {
        event_id(&self.run_id, self.generation, self.sequence)
    }

    pub fn resumed(&self) -> Result<Self, AgentError> {
        let generation = self
            .generation
            .checked_add(1)
            .ok_or_else(|| AgentError::invalid_event("cursor generation overflow"))?;
        let sequence = self
            .sequence
            .checked_add(1)
            .ok_or_else(|| AgentError::invalid_event("cursor sequence overflow"))?;
        Ok(Self {
            project_id: self.project_id.clone(),
            task_id: self.task_id.clone(),
            run_id: self.run_id.clone(),
            generation,
            sequence,
        })
    }

    pub fn check_event(&self, event: &AgentEvent) -> Result<(), AgentError> {
        if self.project_id != event.project_id
            || self.task_id != event.task_id
            || self.run_id != event.run_id
        {
            return Err(AgentError::new(
                AgentErrorCode::CrossTaskEvent,
                "event belongs to a different project, task or run",
            ));
        }
        if self.generation != event.generation {
            return Err(AgentError::new(
                AgentErrorCode::StaleGeneration,
                "event generation does not match the cursor generation",
            ));
        }
        let expected_sequence = self.sequence.checked_add(1).ok_or_else(|| {
            AgentError::new(AgentErrorCode::OutOfOrder, "cursor sequence overflow")
        })?;
        if event.sequence != expected_sequence {
            return Err(AgentError::new(
                AgentErrorCode::OutOfOrder,
                format!("event sequence must be {expected_sequence} after the cursor position"),
            ));
        }
        Ok(())
    }

    pub fn check_resume(&self, event: &AgentEvent) -> Result<Self, AgentError> {
        if self.project_id != event.project_id
            || self.task_id != event.task_id
            || self.run_id != event.run_id
        {
            return Err(AgentError::new(
                AgentErrorCode::CrossTaskEvent,
                "resume event belongs to a different project, task or run",
            ));
        }
        if event.event_type != "run.resumed" {
            return Err(AgentError::invalid_event(
                "check_resume requires a run.resumed event",
            ));
        }
        if self.sequence == 0 {
            return Err(AgentError::invalid_event(
                "resume requires a verified run.started cursor",
            ));
        }
        let expected_generation = self
            .generation
            .checked_add(1)
            .ok_or_else(|| AgentError::invalid_event("cursor generation overflow"))?;
        if event.generation != expected_generation {
            return Err(AgentError::new(
                AgentErrorCode::StaleGeneration,
                format!("run.resumed generation must be {expected_generation}"),
            ));
        }
        let expected_sequence = self.sequence.checked_add(1).ok_or_else(|| {
            AgentError::new(AgentErrorCode::OutOfOrder, "cursor sequence overflow")
        })?;
        if event.sequence != expected_sequence {
            return Err(AgentError::new(
                AgentErrorCode::OutOfOrder,
                format!("run.resumed sequence must be {expected_sequence}"),
            ));
        }
        let data = event
            .data
            .as_object()
            .ok_or_else(|| AgentError::invalid_event("run.resumed data must be an object"))?;
        let previous_generation = data
            .get("previous_generation")
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                AgentError::invalid_event("run.resumed requires data.previous_generation")
            })?;
        if previous_generation != self.generation {
            return Err(AgentError::invalid_event(format!(
                "run.resumed previous_generation must be {}",
                self.generation
            )));
        }
        let snapshot_digest = data
            .get("snapshot_digest")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AgentError::invalid_event("run.resumed requires data.snapshot_digest")
            })?;
        if !is_digest_string(snapshot_digest) {
            return Err(AgentError::invalid_event(
                "run.resumed snapshot_digest must be sha256:<64 lowercase hex>",
            ));
        }
        let resumed_at_ms = data
            .get("resumed_at_ms")
            .and_then(Value::as_u64)
            .ok_or_else(|| AgentError::invalid_event("run.resumed requires data.resumed_at_ms"))?;
        if resumed_at_ms > MAX_SAFE_INTEGER {
            return Err(AgentError::invalid_event(
                "run.resumed resumed_at_ms exceeds the safe integer range",
            ));
        }
        Ok(Self {
            project_id: event.project_id.clone(),
            task_id: event.task_id.clone(),
            run_id: event.run_id.clone(),
            generation: event.generation,
            sequence: event.sequence,
        })
    }
}

pub fn schema_document() -> String {
    stable_json(&build_schema_document())
}

fn build_schema_document() -> Value {
    let run_states: Vec<&str> = RunState::ALL.iter().map(|state| state.as_str()).collect();
    let error_codes: Vec<&str> = AgentErrorCode::ALL
        .iter()
        .map(|code| code.as_str())
        .collect();

    let envelope = object_schema(
        &[
            "schema",
            "event_id",
            "project_id",
            "task_id",
            "run_id",
            "generation",
            "sequence",
            "recorded_at_ms",
            "type",
            "data",
        ],
        json!({
            "schema": {"const": AGENT_EVENT_SCHEMA},
            "event_id": {"type": "string", "pattern": "^evt_run_[0-9a-f]{16}_[0-9]+_[0-9]{12,}$"},
            "project_id": {"type": "string", "pattern": "^prj_[0-9a-f]{16}$"},
            "task_id": {"type": "string", "pattern": "^tsk_[0-9a-f]{16}$"},
            "run_id": {"type": "string", "pattern": "^run_[0-9a-f]{16}$"},
            "generation": positive_integer_schema(),
            "sequence": safe_integer_schema(),
            "recorded_at_ms": safe_integer_schema(),
            "type": {"type": "string", "pattern": "^[a-z][a-z0-9_-]*(\\.[a-z0-9_-]+)*$"},
            "data": {"type": "object"},
        }),
    );

    let model_schema = object_schema(
        &["id", "kind"],
        json!({
            "id": non_empty_string_schema(),
            "kind": non_empty_string_schema(),
        }),
    );
    let environment_schema = object_schema(
        &["kind"],
        json!({
            "kind": non_empty_string_schema(),
        }),
    );
    let step_schema = object_schema(
        &["step_id", "label", "state"],
        json!({
            "step_id": non_empty_string_schema(),
            "label": string_schema(),
            "state": enum_schema(&["pending", "active", "done", "blocked"]),
        }),
    );

    let event_types = json!({
        "run.queued": event_type_schema(
            object_schema(&["queued_at_ms"], json!({"queued_at_ms": safe_integer_schema()})),
            false
        ),
        "run.started": event_type_schema(
            object_schema(
                &["started_at_ms", "model", "mode", "environment", "retry_of_run_id"],
                json!({
                    "started_at_ms": safe_integer_schema(),
                    "model": model_schema,
                    "mode": enum_schema(&["plan", "agent"]),
                    "environment": environment_schema,
                    "retry_of_run_id": nullable_run_id_schema(),
                })
            ),
            false
        ),
        "run.status": event_type_schema(
            object_schema(
                &["status"],
                json!({
                    "status": {"type": "string", "enum": run_states.clone()},
                    "detail": string_schema(),
                })
            ),
            false
        ),
        "run.finished": event_type_schema(
            object_schema(
                &["status", "finished_at_ms", "duration_ms"],
                json!({
                    "status": {"type": "string", "const": "finished"},
                    "finished_at_ms": safe_integer_schema(),
                    "duration_ms": safe_integer_schema(),
                })
            ),
            true
        ),
        "run.failed": event_type_schema(
            object_schema(
                &["code", "message", "retryable"],
                json!({
                    "code": non_empty_string_schema(),
                    "message": string_schema(),
                    "retryable": {"type": "boolean"},
                    "stop_outcome": enum_schema(&["ack_timeout", "host_refused"]),
                    "in_flight_tool_id": string_schema(),
                })
            ),
            true
        ),
        "run.cancelled": event_type_schema(
            object_schema(
                &["safe_boundary", "finished_at_ms"],
                json!({
                    "safe_boundary": {"type": "boolean", "const": true},
                    "stopped_at_tool_id": string_schema(),
                    "finished_at_ms": safe_integer_schema(),
                })
            ),
            true
        ),
        "run.expired": event_type_schema(
            object_schema(
                &["expired_at_ms", "reason"],
                json!({
                    "expired_at_ms": safe_integer_schema(),
                    "reason": non_empty_string_schema(),
                })
            ),
            true
        ),
        "run.resumed": event_type_schema(
            object_schema(
                &["resumed_at_ms", "previous_generation", "snapshot_digest"],
                json!({
                    "resumed_at_ms": safe_integer_schema(),
                    "previous_generation": positive_integer_schema(),
                    "snapshot_digest": digest_schema(),
                })
            ),
            false
        ),
        "turn.started": event_type_schema(
            object_schema(
                &["turn_id", "index"],
                json!({
                    "turn_id": non_empty_string_schema(),
                    "index": safe_integer_schema(),
                })
            ),
            false
        ),
        "turn.finished": event_type_schema(
            object_schema(
                &["turn_id", "finish_reason"],
                json!({
                    "turn_id": non_empty_string_schema(),
                    "finish_reason": enum_schema(&["stop", "length", "tool", "cancelled", "error"]),
                })
            ),
            false
        ),
        "assistant.delta": event_type_schema(
            object_schema(
                &["message_id", "text_delta"],
                json!({
                    "message_id": non_empty_string_schema(),
                    "text_delta": string_schema(),
                })
            ),
            false
        ),
        "assistant.message": event_type_schema(
            object_schema(
                &["message_id", "text"],
                json!({
                    "message_id": non_empty_string_schema(),
                    "text": string_schema(),
                })
            ),
            false
        ),
        "thinking.delta": event_type_schema(
            object_schema(
                &["text_delta"],
                json!({"text_delta": string_schema()})
            ),
            false
        ),
        "thinking.message": event_type_schema(
            object_schema(&["text"], json!({"text": string_schema()})),
            false
        ),
        "plan.updated": event_type_schema(
            object_schema(
                &["plan_id", "title", "steps"],
                json!({
                    "plan_id": non_empty_string_schema(),
                    "title": string_schema(),
                    "steps": {"type": "array", "items": step_schema},
                })
            ),
            false
        ),
        "message.queued": event_type_schema(
            object_schema(
                &["message_id", "delivery", "text_preview"],
                json!({
                    "message_id": non_empty_string_schema(),
                    "delivery": enum_schema(&["queue", "steer", "force"]),
                    "boundary": enum_schema(&["next_turn", "next_tool"]),
                    "text_preview": string_schema(),
                })
            ),
            false
        ),
        "message.steered": event_type_schema(
            object_schema(
                &["message_id", "boundary"],
                json!({
                    "message_id": non_empty_string_schema(),
                    "boundary": enum_schema(&["next_tool", "next_turn"]),
                })
            ),
            false
        ),
        "message.forced": event_type_schema(
            object_schema(
                &["message_id", "boundary"],
                json!({
                    "message_id": non_empty_string_schema(),
                    "boundary": enum_schema(&["next_tool", "next_turn"]),
                })
            ),
            false
        ),
        "tool.requested": event_type_schema(
            object_schema(
                &["tool_call_id", "tool", "arguments_digest", "arguments_preview"],
                json!({
                    "tool_call_id": non_empty_string_schema(),
                    "tool": non_empty_string_schema(),
                    "arguments_digest": digest_schema(),
                    "arguments_preview": string_schema(),
                })
            ),
            false
        ),
        "tool.approval_required": event_type_schema(
            object_schema(
                &["approval_id", "tool_call_id", "tool", "scope", "request_digest", "expires_at_ms"],
                json!({
                    "approval_id": non_empty_string_schema(),
                    "tool_call_id": non_empty_string_schema(),
                    "tool": non_empty_string_schema(),
                    "scope": non_empty_string_schema(),
                    "request_digest": digest_schema(),
                    "expires_at_ms": safe_integer_schema(),
                })
            ),
            false
        ),
        "tool.approval_resolved": event_type_schema(
            object_schema(
                &["approval_id", "decision", "resolver"],
                json!({
                    "approval_id": non_empty_string_schema(),
                    "decision": enum_schema(&["approved", "denied", "expired"]),
                    "resolver": {"type": "string", "const": "host"},
                })
            ),
            false
        ),
        "tool.started": event_type_schema(
            object_schema(&["tool_call_id"], json!({"tool_call_id": non_empty_string_schema()})),
            false
        ),
        "tool.completed": event_type_schema(
            object_schema(
                &["tool_call_id", "status", "output_digest", "output_preview", "truncated"],
                json!({
                    "tool_call_id": non_empty_string_schema(),
                    "status": enum_schema(&["ok", "error", "cancelled"]),
                    "output_digest": digest_schema(),
                    "output_preview": string_schema(),
                    "truncated": {"type": "boolean"},
                })
            ),
            false
        ),
        "tool.failed": event_type_schema(
            object_schema(
                &["tool_call_id", "code", "message"],
                json!({
                    "tool_call_id": non_empty_string_schema(),
                    "code": non_empty_string_schema(),
                    "message": string_schema(),
                })
            ),
            false
        ),
        "artifact.created": event_type_schema(
            object_schema(
                &["artifact_id", "kind", "name", "media_type", "size_bytes", "digest"],
                json!({
                    "artifact_id": non_empty_string_schema(),
                    "kind": enum_schema(&["file", "image", "log", "link"]),
                    "name": non_empty_string_schema(),
                    "media_type": non_empty_string_schema(),
                    "size_bytes": safe_integer_schema(),
                    "digest": digest_schema(),
                })
            ),
            false
        ),
        "changes.snapshot": event_type_schema(
            object_schema(
                &["scope", "files", "head_digest", "captured_at_ms"],
                json!({
                    "scope": enum_schema(&["last_turn", "branch"]),
                    "files": {"type": "array", "items": {"type": "object"}},
                    "head_digest": non_empty_string_schema(),
                    "captured_at_ms": safe_integer_schema(),
                })
            ),
            false
        ),
        "diff.comment": event_type_schema(
            object_schema(
                &["comment_id", "path", "line", "body", "head_digest"],
                json!({
                    "comment_id": non_empty_string_schema(),
                    "path": non_empty_string_schema(),
                    "line": safe_integer_schema(),
                    "body": string_schema(),
                    "head_digest": non_empty_string_schema(),
                })
            ),
            false
        ),
        "usage.updated": event_type_schema(
            object_schema(
                &["input_tokens", "output_tokens", "model_calls"],
                json!({
                    "input_tokens": safe_integer_schema(),
                    "output_tokens": safe_integer_schema(),
                    "cache_read_tokens": safe_integer_schema(),
                    "cache_write_tokens": safe_integer_schema(),
                    "model_calls": safe_integer_schema(),
                })
            ),
            false
        ),
        "heartbeat": event_type_schema(
            object_schema(&["nonce"], json!({"nonce": non_empty_string_schema()})),
            false
        ),
        "error": event_type_schema(
            object_schema(
                &["code", "message", "retryable"],
                json!({
                    "code": non_empty_string_schema(),
                    "message": string_schema(),
                    "retryable": {"type": "boolean"},
                })
            ),
            false
        ),
    });

    json!({
        "$id": AGENT_EVENT_SCHEMA,
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "canonical_json_schema": CANONICAL_JSON_SCHEMA,
        "contract_version": 1,
        "cursor_prefix": CURSOR_PREFIX,
        "envelope": envelope,
        "error_codes": error_codes,
        "error_schema": AGENT_ERROR_SCHEMA,
        "event_types": event_types,
        "limits": {
            "max_event_bytes": MAX_EVENT_BYTES,
            "max_events_per_run": MAX_EVENTS_PER_RUN,
            "approval_ttl_ms": APPROVAL_TTL_MS,
        },
        "run_states": run_states,
        "schema_id": AGENT_EVENT_SCHEMA,
        "title": "UOR-R4 Studio agent event"
    })
}

fn validate_known_data(event: &AgentEvent) -> Result<(), AgentError> {
    let event_type = event.event_type.as_str();
    let data = event
        .data
        .as_object()
        .ok_or_else(|| AgentError::invalid_event(format!("{event_type} data must be an object")))?;

    match event_type {
        "run.queued" => {
            expect_sequence(event, 0)?;
            u64_field(data, "queued_at_ms", event_type)?;
        }
        "run.started" => {
            expect_sequence(event, 1)?;
            u64_field(data, "started_at_ms", event_type)?;
            validate_model(data, event_type)?;
            enum_field(data, "mode", event_type, &["plan", "agent"])?;
            validate_environment(data, event_type)?;
            nullable_run_id_field(data, "retry_of_run_id", event_type)?;
        }
        "run.status" => {
            let status = non_empty_string_field(data, "status", event_type)?;
            if RunState::parse(status).is_none() {
                return Err(AgentError::invalid_event(format!(
                    "run.status data.status {status:?} is not a run state"
                )));
            }
            let _ = optional_string_field(data, "detail", event_type)?;
        }
        "run.finished" => {
            if non_empty_string_field(data, "status", event_type)? != "finished" {
                return Err(AgentError::invalid_event(
                    "run.finished data.status must be \"finished\"",
                ));
            }
            u64_field(data, "finished_at_ms", event_type)?;
            u64_field(data, "duration_ms", event_type)?;
        }
        "run.failed" => {
            non_empty_string_field(data, "code", event_type)?;
            string_field(data, "message", event_type)?;
            bool_field(data, "retryable", event_type)?;
            optional_enum_field(
                data,
                "stop_outcome",
                event_type,
                &["ack_timeout", "host_refused"],
            )?;
            let _ = optional_string_field(data, "in_flight_tool_id", event_type)?;
        }
        "run.cancelled" => {
            if !bool_field(data, "safe_boundary", event_type)? {
                return Err(AgentError::invalid_event(
                    "run.cancelled requires data.safe_boundary = true",
                ));
            }
            let _ = optional_string_field(data, "stopped_at_tool_id", event_type)?;
            u64_field(data, "finished_at_ms", event_type)?;
        }
        "run.expired" => {
            u64_field(data, "expired_at_ms", event_type)?;
            non_empty_string_field(data, "reason", event_type)?;
        }
        "run.resumed" => {
            u64_field(data, "resumed_at_ms", event_type)?;
            let previous_generation = u64_field(data, "previous_generation", event_type)?;
            if previous_generation == 0
                || previous_generation.checked_add(1) != Some(event.generation)
            {
                return Err(AgentError::invalid_event(
                    "run.resumed previous_generation must be the verified generation immediately before the current one",
                ));
            }
            if event.sequence < 2 {
                return Err(AgentError::invalid_event(
                    "run.resumed must follow the verified run.queued/run.started markers",
                ));
            }
            digest_field(data, "snapshot_digest", event_type)?;
        }
        "turn.started" => {
            non_empty_string_field(data, "turn_id", event_type)?;
            u64_field(data, "index", event_type)?;
        }
        "turn.finished" => {
            non_empty_string_field(data, "turn_id", event_type)?;
            enum_field(
                data,
                "finish_reason",
                event_type,
                &["stop", "length", "tool", "cancelled", "error"],
            )?;
        }
        "assistant.delta" => {
            non_empty_string_field(data, "message_id", event_type)?;
            string_field(data, "text_delta", event_type)?;
        }
        "assistant.message" => {
            non_empty_string_field(data, "message_id", event_type)?;
            string_field(data, "text", event_type)?;
        }
        "thinking.delta" => {
            string_field(data, "text_delta", event_type)?;
        }
        "thinking.message" => {
            string_field(data, "text", event_type)?;
        }
        "plan.updated" => {
            non_empty_string_field(data, "plan_id", event_type)?;
            string_field(data, "title", event_type)?;
            validate_steps(data, event_type)?;
        }
        "message.queued" => {
            non_empty_string_field(data, "message_id", event_type)?;
            enum_field(data, "delivery", event_type, &["queue", "steer", "force"])?;
            optional_enum_field(data, "boundary", event_type, &["next_turn", "next_tool"])?;
            string_field(data, "text_preview", event_type)?;
        }
        "message.steered" | "message.forced" => {
            non_empty_string_field(data, "message_id", event_type)?;
            enum_field(data, "boundary", event_type, &["next_tool", "next_turn"])?;
        }
        "tool.requested" => {
            non_empty_string_field(data, "tool_call_id", event_type)?;
            non_empty_string_field(data, "tool", event_type)?;
            digest_field(data, "arguments_digest", event_type)?;
            string_field(data, "arguments_preview", event_type)?;
        }
        "tool.approval_required" => {
            non_empty_string_field(data, "approval_id", event_type)?;
            non_empty_string_field(data, "tool_call_id", event_type)?;
            non_empty_string_field(data, "tool", event_type)?;
            non_empty_string_field(data, "scope", event_type)?;
            digest_field(data, "request_digest", event_type)?;
            u64_field(data, "expires_at_ms", event_type)?;
        }
        "tool.approval_resolved" => {
            non_empty_string_field(data, "approval_id", event_type)?;
            enum_field(
                data,
                "decision",
                event_type,
                &["approved", "denied", "expired"],
            )?;
            if non_empty_string_field(data, "resolver", event_type)? != "host" {
                return Err(AgentError::invalid_event(
                    "tool.approval_resolved resolver must be \"host\"",
                ));
            }
        }
        "tool.started" => {
            non_empty_string_field(data, "tool_call_id", event_type)?;
        }
        "tool.completed" => {
            non_empty_string_field(data, "tool_call_id", event_type)?;
            enum_field(data, "status", event_type, &["ok", "error", "cancelled"])?;
            digest_field(data, "output_digest", event_type)?;
            string_field(data, "output_preview", event_type)?;
            bool_field(data, "truncated", event_type)?;
        }
        "tool.failed" => {
            non_empty_string_field(data, "tool_call_id", event_type)?;
            non_empty_string_field(data, "code", event_type)?;
            string_field(data, "message", event_type)?;
        }
        "artifact.created" => {
            non_empty_string_field(data, "artifact_id", event_type)?;
            enum_field(data, "kind", event_type, &["file", "image", "log", "link"])?;
            non_empty_string_field(data, "name", event_type)?;
            non_empty_string_field(data, "media_type", event_type)?;
            u64_field(data, "size_bytes", event_type)?;
            digest_field(data, "digest", event_type)?;
        }
        "changes.snapshot" => {
            enum_field(data, "scope", event_type, &["last_turn", "branch"])?;
            array_field(data, "files", event_type)?;
            non_empty_string_field(data, "head_digest", event_type)?;
            u64_field(data, "captured_at_ms", event_type)?;
        }
        "diff.comment" => {
            non_empty_string_field(data, "comment_id", event_type)?;
            non_empty_string_field(data, "path", event_type)?;
            u64_field(data, "line", event_type)?;
            string_field(data, "body", event_type)?;
            non_empty_string_field(data, "head_digest", event_type)?;
        }
        "usage.updated" => {
            u64_field(data, "input_tokens", event_type)?;
            u64_field(data, "output_tokens", event_type)?;
            u64_field(data, "model_calls", event_type)?;
            let _ = optional_u64_field(data, "cache_read_tokens", event_type)?;
            let _ = optional_u64_field(data, "cache_write_tokens", event_type)?;
        }
        "heartbeat" => {
            non_empty_string_field(data, "nonce", event_type)?;
        }
        "error" => {
            non_empty_string_field(data, "code", event_type)?;
            string_field(data, "message", event_type)?;
            bool_field(data, "retryable", event_type)?;
        }
        _ => {}
    }
    Ok(())
}

fn validate_model(data: &Map<String, Value>, event_type: &str) -> Result<(), AgentError> {
    let model = object_field(data, "model", event_type)?;
    non_empty_string_field(model, "id", event_type)?;
    non_empty_string_field(model, "kind", event_type)?;
    Ok(())
}

fn validate_environment(data: &Map<String, Value>, event_type: &str) -> Result<(), AgentError> {
    let environment = object_field(data, "environment", event_type)?;
    non_empty_string_field(environment, "kind", event_type)?;
    Ok(())
}

fn validate_steps(data: &Map<String, Value>, event_type: &str) -> Result<(), AgentError> {
    let steps = array_field(data, "steps", event_type)?;
    for step in steps {
        let step = step.as_object().ok_or_else(|| {
            AgentError::invalid_event(format!("{event_type} data.steps items must be objects"))
        })?;
        non_empty_string_field(step, "step_id", event_type)?;
        string_field(step, "label", event_type)?;
        enum_field(
            step,
            "state",
            event_type,
            &["pending", "active", "done", "blocked"],
        )?;
    }
    Ok(())
}

fn expect_sequence(event: &AgentEvent, expected: u64) -> Result<(), AgentError> {
    if event.sequence != expected {
        return Err(AgentError::invalid_event(format!(
            "{} must be sequence {expected} in a fresh run",
            event.event_type
        )));
    }
    Ok(())
}

fn required<'a>(
    data: &'a Map<String, Value>,
    key: &str,
    event_type: &str,
) -> Result<&'a Value, AgentError> {
    data.get(key)
        .ok_or_else(|| AgentError::invalid_event(format!("{event_type} requires data.{key}")))
}

fn string_field<'a>(
    data: &'a Map<String, Value>,
    key: &str,
    event_type: &str,
) -> Result<&'a str, AgentError> {
    required(data, key, event_type)?.as_str().ok_or_else(|| {
        AgentError::invalid_event(format!("{event_type} data.{key} must be a string"))
    })
}

fn non_empty_string_field<'a>(
    data: &'a Map<String, Value>,
    key: &str,
    event_type: &str,
) -> Result<&'a str, AgentError> {
    let value = string_field(data, key, event_type)?;
    if value.is_empty() {
        return Err(AgentError::invalid_event(format!(
            "{event_type} data.{key} must not be empty"
        )));
    }
    Ok(value)
}

fn optional_string_field<'a>(
    data: &'a Map<String, Value>,
    key: &str,
    event_type: &str,
) -> Result<Option<&'a str>, AgentError> {
    match data.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.as_str())),
        Some(_) => Err(AgentError::invalid_event(format!(
            "{event_type} data.{key} must be a string when present"
        ))),
    }
}

fn u64_field(data: &Map<String, Value>, key: &str, event_type: &str) -> Result<u64, AgentError> {
    let value = required(data, key, event_type)?.as_u64().ok_or_else(|| {
        AgentError::invalid_event(format!(
            "{event_type} data.{key} must be a non-negative integer"
        ))
    })?;
    if value > MAX_SAFE_INTEGER {
        return Err(AgentError::invalid_event(format!(
            "{event_type} data.{key} exceeds the safe integer range"
        )));
    }
    Ok(value)
}

fn optional_u64_field(
    data: &Map<String, Value>,
    key: &str,
    event_type: &str,
) -> Result<Option<u64>, AgentError> {
    match data.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => u64_field(data, key, event_type).map(Some),
    }
}

fn bool_field(data: &Map<String, Value>, key: &str, event_type: &str) -> Result<bool, AgentError> {
    required(data, key, event_type)?.as_bool().ok_or_else(|| {
        AgentError::invalid_event(format!("{event_type} data.{key} must be a boolean"))
    })
}

fn object_field<'a>(
    data: &'a Map<String, Value>,
    key: &str,
    event_type: &str,
) -> Result<&'a Map<String, Value>, AgentError> {
    required(data, key, event_type)?.as_object().ok_or_else(|| {
        AgentError::invalid_event(format!("{event_type} data.{key} must be an object"))
    })
}

fn array_field<'a>(
    data: &'a Map<String, Value>,
    key: &str,
    event_type: &str,
) -> Result<&'a Vec<Value>, AgentError> {
    required(data, key, event_type)?.as_array().ok_or_else(|| {
        AgentError::invalid_event(format!("{event_type} data.{key} must be an array"))
    })
}

fn enum_field<'a>(
    data: &'a Map<String, Value>,
    key: &str,
    event_type: &str,
    allowed: &[&str],
) -> Result<&'a str, AgentError> {
    let value = string_field(data, key, event_type)?;
    if !allowed.contains(&value) {
        return Err(AgentError::invalid_event(format!(
            "{event_type} data.{key} {value:?} is not one of {allowed:?}"
        )));
    }
    Ok(value)
}

fn optional_enum_field(
    data: &Map<String, Value>,
    key: &str,
    event_type: &str,
    allowed: &[&str],
) -> Result<(), AgentError> {
    if optional_string_field(data, key, event_type)?.is_some() {
        enum_field(data, key, event_type, allowed)?;
    }
    Ok(())
}

fn digest_field(data: &Map<String, Value>, key: &str, event_type: &str) -> Result<(), AgentError> {
    if !is_digest_string(string_field(data, key, event_type)?) {
        return Err(AgentError::invalid_event(format!(
            "{event_type} data.{key} must be sha256:<64 lowercase hex>"
        )));
    }
    Ok(())
}

fn nullable_run_id_field(
    data: &Map<String, Value>,
    key: &str,
    event_type: &str,
) -> Result<(), AgentError> {
    match required(data, key, event_type)? {
        Value::Null => Ok(()),
        Value::String(run_id) if is_valid_id(run_id, "run") => Ok(()),
        _ => Err(AgentError::invalid_event(format!(
            "{event_type} data.{key} must be null or a run_<16 hex> id"
        ))),
    }
}

fn is_event_type_name(value: &str) -> bool {
    let mut parts = value.split('.');
    let Some(first) = parts.next() else {
        return false;
    };
    let Some(first_character) = first.chars().next() else {
        return false;
    };
    if !first_character.is_ascii_lowercase() || !first.chars().all(is_event_type_character) {
        return false;
    }
    for part in parts {
        if part.is_empty() || !part.chars().all(is_event_type_character) {
            return false;
        }
    }
    true
}

fn is_event_type_character(character: char) -> bool {
    character.is_ascii_lowercase()
        || character.is_ascii_digit()
        || character == '_'
        || character == '-'
}

fn b64url_encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity((bytes.len() * 4).div_ceil(3));
    let mut accumulator = 0u32;
    let mut bits = 0u32;
    for &byte in bytes {
        accumulator = (accumulator << 8) | u32::from(byte);
        bits += 8;
        while bits >= 6 {
            bits -= 6;
            out.push(ALPHABET[((accumulator >> bits) & 0x3f) as usize] as char);
            accumulator &= (1u32 << bits) - 1;
        }
    }
    if bits > 0 {
        out.push(ALPHABET[((accumulator << (6 - bits)) & 0x3f) as usize] as char);
    }
    out
}

fn b64url_decode(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len() * 3 / 4);
    let mut accumulator = 0u32;
    let mut bits = 0u32;
    for byte in text.bytes() {
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'-' => 62,
            b'_' => 63,
            _ => return None,
        };
        accumulator = (accumulator << 6) | u32::from(value);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((accumulator >> bits) & 0xff) as u8);
            accumulator &= (1u32 << bits) - 1;
        }
    }
    if bits >= 6 {
        return None;
    }
    if bits > 0 && accumulator != 0 {
        return None;
    }
    Some(out)
}

fn decode_id(encoded: &str, prefix: &str) -> Result<String, AgentError> {
    let bytes = b64url_decode(encoded)
        .ok_or_else(|| AgentError::invalid_event("cursor id is not unpadded base64url"))?;
    let value = String::from_utf8(bytes)
        .map_err(|_| AgentError::invalid_event("cursor id is not UTF-8"))?;
    if !is_valid_id(&value, prefix) {
        return Err(AgentError::invalid_event(format!(
            "cursor id is not a {prefix}_<16 hex> id"
        )));
    }
    Ok(value)
}

fn parse_decimal(value: &str, field: &str) -> Result<u64, AgentError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(AgentError::invalid_event(format!(
            "cursor {field} must be decimal"
        )));
    }
    value
        .parse::<u64>()
        .map_err(|_| AgentError::invalid_event(format!("cursor {field} is out of range")))
}

fn safe_integer_schema() -> Value {
    json!({"type": "integer", "minimum": 0, "maximum": MAX_SAFE_INTEGER})
}

fn positive_integer_schema() -> Value {
    json!({"type": "integer", "minimum": 1, "maximum": MAX_SAFE_INTEGER})
}

fn string_schema() -> Value {
    json!({"type": "string"})
}

fn non_empty_string_schema() -> Value {
    json!({"type": "string", "minLength": 1})
}

fn digest_schema() -> Value {
    json!({"type": "string", "pattern": "^sha256:[0-9a-f]{64}$"})
}

fn enum_schema(values: &[&str]) -> Value {
    json!({"type": "string", "enum": values})
}

fn nullable_run_id_schema() -> Value {
    json!({"type": ["string", "null"], "pattern": "^run_[0-9a-f]{16}$"})
}

fn object_schema(required: &[&str], properties: Value) -> Value {
    json!({"type": "object", "required": required, "properties": properties})
}

fn event_type_schema(data: Value, terminal: bool) -> Value {
    json!({"data": data, "terminal": terminal})
}
