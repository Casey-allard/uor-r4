//! Stable error codes and the error envelope from spec §9.6.

use serde::{Deserialize, Serialize};

use crate::AGENT_ERROR_SCHEMA;

/// Error codes shared by the Rust contract and the generated TypeScript client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentErrorCode {
    InvalidEvent,
    InvalidArguments,
    MalformedEvent,
    TruncatedFrame,
    DuplicateEvent,
    OutOfOrder,
    StaleGeneration,
    CrossTaskEvent,
    CursorExpired,
    SnapshotCorrupt,
    SnapshotMissing,
    ApprovalForbidden,
    ApprovalExpired,
    ApprovalDigestMismatch,
    TaskNotFound,
    RunNotFound,
    TaskArchived,
    RunNotRetryable,
    StopUnacknowledged,
    RuntimeUnavailable,
    HostUnavailable,
    HostDenied,
    ModelUnavailable,
    ModelProtocolError,
    QuotaExceeded,
    TransportClosed,
    LimitExceeded,
    Internal,
}

impl AgentErrorCode {
    pub const ALL: [AgentErrorCode; 28] = [
        AgentErrorCode::InvalidEvent,
        AgentErrorCode::InvalidArguments,
        AgentErrorCode::MalformedEvent,
        AgentErrorCode::TruncatedFrame,
        AgentErrorCode::DuplicateEvent,
        AgentErrorCode::OutOfOrder,
        AgentErrorCode::StaleGeneration,
        AgentErrorCode::CrossTaskEvent,
        AgentErrorCode::CursorExpired,
        AgentErrorCode::SnapshotCorrupt,
        AgentErrorCode::SnapshotMissing,
        AgentErrorCode::ApprovalForbidden,
        AgentErrorCode::ApprovalExpired,
        AgentErrorCode::ApprovalDigestMismatch,
        AgentErrorCode::TaskNotFound,
        AgentErrorCode::RunNotFound,
        AgentErrorCode::TaskArchived,
        AgentErrorCode::RunNotRetryable,
        AgentErrorCode::StopUnacknowledged,
        AgentErrorCode::RuntimeUnavailable,
        AgentErrorCode::HostUnavailable,
        AgentErrorCode::HostDenied,
        AgentErrorCode::ModelUnavailable,
        AgentErrorCode::ModelProtocolError,
        AgentErrorCode::QuotaExceeded,
        AgentErrorCode::TransportClosed,
        AgentErrorCode::LimitExceeded,
        AgentErrorCode::Internal,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            AgentErrorCode::InvalidEvent => "invalid_event",
            AgentErrorCode::InvalidArguments => "invalid_arguments",
            AgentErrorCode::MalformedEvent => "malformed_event",
            AgentErrorCode::TruncatedFrame => "truncated_frame",
            AgentErrorCode::DuplicateEvent => "duplicate_event",
            AgentErrorCode::OutOfOrder => "out_of_order",
            AgentErrorCode::StaleGeneration => "stale_generation",
            AgentErrorCode::CrossTaskEvent => "cross_task_event",
            AgentErrorCode::CursorExpired => "cursor_expired",
            AgentErrorCode::SnapshotCorrupt => "snapshot_corrupt",
            AgentErrorCode::SnapshotMissing => "snapshot_missing",
            AgentErrorCode::ApprovalForbidden => "approval_forbidden",
            AgentErrorCode::ApprovalExpired => "approval_expired",
            AgentErrorCode::ApprovalDigestMismatch => "approval_digest_mismatch",
            AgentErrorCode::TaskNotFound => "task_not_found",
            AgentErrorCode::RunNotFound => "run_not_found",
            AgentErrorCode::TaskArchived => "task_archived",
            AgentErrorCode::RunNotRetryable => "run_not_retryable",
            AgentErrorCode::StopUnacknowledged => "stop_unacknowledged",
            AgentErrorCode::RuntimeUnavailable => "runtime_unavailable",
            AgentErrorCode::HostUnavailable => "host_unavailable",
            AgentErrorCode::HostDenied => "host_denied",
            AgentErrorCode::ModelUnavailable => "model_unavailable",
            AgentErrorCode::ModelProtocolError => "model_protocol_error",
            AgentErrorCode::QuotaExceeded => "quota_exceeded",
            AgentErrorCode::TransportClosed => "transport_closed",
            AgentErrorCode::LimitExceeded => "limit_exceeded",
            AgentErrorCode::Internal => "internal",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        AgentErrorCode::ALL
            .iter()
            .copied()
            .find(|code| code.as_str() == value)
    }
}

/// The `uor-r4.studio.agent-error/1` envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentError {
    pub schema: String,
    pub code: AgentErrorCode,
    pub message: String,
    pub retryable: bool,
    pub task_id: Option<String>,
    pub run_id: Option<String>,
}

impl AgentError {
    pub fn new(code: AgentErrorCode, message: impl Into<String>) -> Self {
        Self {
            schema: AGENT_ERROR_SCHEMA.to_string(),
            code,
            message: message.into(),
            retryable: false,
            task_id: None,
            run_id: None,
        }
    }

    pub fn invalid_event(message: impl Into<String>) -> Self {
        Self::new(AgentErrorCode::InvalidEvent, message)
    }

    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }

    pub fn with_scope(mut self, task_id: Option<String>, run_id: Option<String>) -> Self {
        self.task_id = task_id;
        self.run_id = run_id;
        self
    }

    pub fn code(&self) -> &'static str {
        self.code.as_str()
    }
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for AgentError {}
