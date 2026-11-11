//! Browser-owned Studio agent contract (issue #3917).
//!
//! This crate owns the Rust reference for the `uor-r4.studio.agent-event/1`
//! envelope, `uor-r4.canonical-json/1` and the schema exported to the browser
//! client. It contains no runtime, transport, host store or model code.

#![forbid(unsafe_code)]

pub mod canonical;
pub mod contract;
pub mod error;
pub mod ids;

pub const AGENT_EVENT_SCHEMA: &str = "uor-r4.studio.agent-event/1";
pub const AGENT_ERROR_SCHEMA: &str = "uor-r4.studio.agent-error/1";
pub const CANONICAL_JSON_SCHEMA: &str = "uor-r4.canonical-json/1";
pub const CURSOR_PREFIX: &str = "uorse1";

pub const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
pub const MAX_EVENT_BYTES: u64 = 1_048_576;
pub const MAX_EVENTS_PER_RUN: u64 = 10_000;
pub const APPROVAL_TTL_MS: u64 = 600_000;

pub use canonical::{canonical_json, digest_event, parse_canonical_json};
pub use contract::{
    is_known_event_type, schema_document, validate_approval, validate_event,
    validate_run_start_pair, ActiveRun, AgentCursor, AgentEvent, ApprovalRequest, RunState,
    TaskState, TaskSummary, KNOWN_EVENT_TYPES,
};
pub use error::{AgentError, AgentErrorCode};
pub use ids::{
    event_id, is_valid_id, new_approval_id, new_artifact_id, new_message_id, new_project_id,
    new_run_id, new_task_id, new_tool_call_id, snapshot_id, turn_id,
};
