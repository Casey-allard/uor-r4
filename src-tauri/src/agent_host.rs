//! Approved companion tool intents for the Studio agent host.
//!
//! The browser page owns task, run, event, approval and cursor state. This
//! module executes only a canonical [`HostToolIntent`] whose approval identity
//! was issued and confirmed by the host itself, and reports a
//! [`HostToolResult`]. Operational result queues and subscriptions are bounded
//! transport data, not a task log or a second runtime.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use uor_r4_studio_agent::{canonical_json, is_valid_id};

use crate::agent_data::{AgentArtifactRange, MAX_AGENT_ARTIFACT_BYTES};
use crate::{
    reject_parent_dir_traversal, sha256_bytes, DesktopBackend, DesktopClientError,
    MAX_WRITE_FILE_BYTES,
};

/// Schema identity of a page → host tool intent.
pub const HOST_INTENT_SCHEMA: &str = "uor-r4.studio.host-intent/1";
/// Schema identity of a host → page tool result.
pub const HOST_RESULT_SCHEMA: &str = "uor-r4.studio.host-result/1";
/// Host approval lifetime from request to decision/execution.
pub const HOST_APPROVAL_TTL_MS: u64 = 600_000;
/// Maximum retained transport results.
pub const MAX_HOST_RESULTS: usize = 256;
/// Maximum simultaneously pending or approved intents.
pub const MAX_PENDING_APPROVALS: usize = 128;
/// Maximum live Channel/poll subscribers retained by one host.
pub const MAX_HOST_SUBSCRIBERS: usize = 8;
/// Maximum canonical argument JSON accepted for one intent.
pub const MAX_HOST_ARGUMENT_BYTES: usize = 20 * 1024 * 1024;
/// Maximum characters in a result preview.
pub const MAX_HOST_PREVIEW_CHARS: usize = 512;
/// Maximum characters in a host approval's path/argument preview.
pub const MAX_APPROVAL_PREVIEW_CHARS: usize = 384;

const HOST_RECEIPT_DOMAIN: &str = "uor-r4.studio.host-receipt/1\n";
const DATA_TASK_ID: &str = "tsk_0000000000000000";
const DATA_RUN_ID: &str = "run_0000000000000000";
const AUTHORITY_FIELDS: [&str; 9] = [
    "granted",
    "scope",
    "command",
    "key",
    "receipt",
    "path",
    "origin",
    "resolved_by",
    "grants",
];
const ARGUMENT_AUTHORITY_FIELDS: [&str; 6] = [
    "granted",
    "receipt",
    "origin",
    "resolved_by",
    "secret",
    "preview",
];

/// Optional artifact reference carried beside one host result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostArtifactRef {
    pub artifact_id: String,
    pub digest: String,
    pub media_type: String,
    pub size_bytes: u64,
}

/// One host tool outcome. This is not an AgentEvent and carries no event,
/// sequence or task-cursor fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostToolResult {
    pub schema: String,
    pub intent_id: String,
    pub task_id: String,
    pub run_id: String,
    pub generation: u64,
    pub tool_call_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_scope: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<Value>,
    pub output_digest: String,
    pub output_preview: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<HostArtifactRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_receipt: Option<String>,
    pub occurred_at_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Native confirmation text built only from host-held approval data.
pub fn approval_dialog_message(summary: &HostApprovalSummary) -> String {
    format!(
        "Approve this host tool intent?\n\nTool:\n{}\n\nPreview:\n{}\n\nTask:\n{}\n\nRun:\n{}\n\nTool call:\n{}\n\nRequest digest:\n{}\n\nExpires at (ms):\n{}",
        summary.tool,
        summary.preview,
        summary.task_id,
        summary.run_id,
        summary.tool_call_id,
        summary.request_digest,
        summary.expires_at_ms,
    )
}

/// Host-held description of one approval request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostApprovalSummary {
    pub approval_id: String,
    pub task_id: String,
    pub run_id: String,
    pub generation: u64,
    pub tool_call_id: String,
    pub tool: String,
    pub scope: String,
    pub preview: String,
    pub request_digest: String,
    pub requested_at_ms: u64,
    pub expires_at_ms: u64,
    pub approved: bool,
    pub resolved: bool,
}

/// Host-owned MCP dispatch boundary. Only an already host-confirmed session is
/// reachable through [`AgentHost`]; the transport never registers a session.
pub trait HostMcpDispatch {
    fn list_tools(&self, session_id: &str) -> Result<Value, String>;
    fn call_tool(&self, session_id: &str, tool: &str, arguments: &Value) -> Result<Value, String>;
}

impl HostMcpDispatch for crate::mcp::McpManager {
    fn list_tools(&self, session_id: &str) -> Result<Value, String> {
        crate::mcp::McpManager::list_tools(self, session_id).map(|tools| json!(tools))
    }

    fn call_tool(&self, session_id: &str, tool: &str, arguments: &Value) -> Result<Value, String> {
        crate::mcp::McpManager::call_tool(self, session_id, tool, arguments)
    }
}

/// Explicit refusal for a host that has no MCP manager (offline shim).
pub struct UnavailableMcpDispatch;

impl HostMcpDispatch for UnavailableMcpDispatch {
    fn list_tools(&self, _session_id: &str) -> Result<Value, String> {
        Err("MCP sessions are host-owned and unavailable in this host".to_string())
    }

    fn call_tool(
        &self,
        _session_id: &str,
        _tool: &str,
        _arguments: &Value,
    ) -> Result<Value, String> {
        Err("MCP sessions are host-owned and unavailable in this host".to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToolKind {
    Tree,
    Changes,
    GitStatus,
    ArtifactRead,
    FileRead,
    FileWrite,
    TerminalRun,
    TerminalOpen,
    TerminalInput,
    McpCall,
}

impl ToolKind {
    fn for_name(tool: &str) -> Option<Self> {
        match tool {
            "file.tree" => Some(Self::Tree),
            "git.changes" => Some(Self::Changes),
            "git.status" => Some(Self::GitStatus),
            "artifact.read" => Some(Self::ArtifactRead),
            "file.read" => Some(Self::FileRead),
            "file.write" => Some(Self::FileWrite),
            "terminal.run" => Some(Self::TerminalRun),
            "terminal.open" => Some(Self::TerminalOpen),
            "terminal.input" => Some(Self::TerminalInput),
            "mcp.call" => Some(Self::McpCall),
            _ => None,
        }
    }

    fn is_data(self) -> bool {
        matches!(
            self,
            Self::Tree | Self::Changes | Self::GitStatus | Self::ArtifactRead
        )
    }

    fn needs_approval(self) -> bool {
        matches!(self, Self::FileRead | Self::FileWrite)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedIntent {
    intent_id: String,
    task_id: String,
    run_id: String,
    generation: u64,
    tool_call_id: String,
    tool: String,
    arguments: Value,
    arguments_digest: String,
    request_scope: String,
    approval_id: Option<String>,
}

#[derive(Debug, Clone)]
struct HostFailure {
    code: &'static str,
    message: String,
}

impl HostFailure {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    fn invalid_arguments(message: impl Into<String>) -> Self {
        Self::new("invalid_arguments", message)
    }

    fn approval_forbidden(message: impl Into<String>) -> Self {
        Self::new("approval_forbidden", message)
    }

    fn host_denied(message: impl Into<String>) -> Self {
        Self::new("host_denied", message)
    }
}

fn backend_failure(error: DesktopClientError) -> HostFailure {
    let code = match &error {
        DesktopClientError::PathTraversalRefused(_)
        | DesktopClientError::PathNotGranted(_)
        | DesktopClientError::SymlinkRefused(_)
        | DesktopClientError::TerminalDisabledByDefault
        | DesktopClientError::TerminalConsentRequired
        | DesktopClientError::TerminalMetacharacterRefused(_)
        | DesktopClientError::SessionNotFound(_) => "host_denied",
        DesktopClientError::FileTooLarge { .. } | DesktopClientError::SessionLimitReached => {
            "limit_exceeded"
        }
        DesktopClientError::ArtifactDigestMismatch { .. }
        | DesktopClientError::ArtifactSizeMismatch { .. } => "invalid_event",
        DesktopClientError::InvalidInput(_)
        | DesktopClientError::ArtifactNotFound(_)
        | DesktopClientError::UnsupportedModelFormat(_) => "invalid_arguments",
        _ => "internal",
    };
    HostFailure::new(code, error.to_string())
}

fn internal_failure(message: impl Into<String>) -> HostFailure {
    HostFailure::new("internal", message)
}

/// Canonical error-code prefix shared by the host and its HTTP transport.
pub fn error_code(error: &str) -> &'static str {
    match error.split(':').next().unwrap_or("").trim() {
        "invalid_arguments" => "invalid_arguments",
        "approval_forbidden" => "approval_forbidden",
        "approval_expired" => "approval_expired",
        "approval_digest_mismatch" => "approval_digest_mismatch",
        "cross_task_event" => "cross_task_event",
        "out_of_order" => "out_of_order",
        "stale_generation" => "stale_generation",
        "limit_exceeded" => "limit_exceeded",
        "host_denied" => "host_denied",
        "host_unavailable" => "host_unavailable",
        "cursor_expired" => "cursor_expired",
        _ => "internal",
    }
}

fn sha256_prefixed(bytes: &[u8]) -> String {
    format!("sha256:{}", sha256_bytes(bytes))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn preview(text: &str) -> String {
    text.chars().take(MAX_HOST_PREVIEW_CHARS).collect()
}

/// RFC 2104 HMAC-SHA-256 over `message` with `key`, lowercase hex.
///
/// The receipt key is generated inside [`AgentHost`] and is never serialized;
/// the page only ever sees the resulting opaque `hmac-sha256:` value.
pub fn hmac_sha256_hex(key: &[u8], message: &[u8]) -> String {
    let mut block = [0u8; 64];
    if key.len() > 64 {
        block[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        block[..key.len()].copy_from_slice(key);
    }
    let mut inner_pad = [0x36u8; 64];
    let mut outer_pad = [0x5cu8; 64];
    for index in 0..64 {
        inner_pad[index] ^= block[index];
        outer_pad[index] ^= block[index];
    }
    let mut inner = Sha256::new();
    inner.update(inner_pad);
    inner.update(message);
    let inner = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(outer_pad);
    outer.update(inner);
    hex::encode(outer.finalize())
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0u8;
    for (a, b) in left.iter().zip(right) {
        difference |= a ^ b;
    }
    difference == 0
}

fn escape_approval_character(character: char) -> String {
    match character {
        '\n' => "\\n".to_string(),
        '\r' => "\\r".to_string(),
        '\t' => "\\t".to_string(),
        control if control.is_control() => format!("\\u{{{:04x}}}", control as u32),
        other => other.to_string(),
    }
}

fn escaped_approval_len(text: &str) -> usize {
    text.chars()
        .map(|character| match character {
            '\n' | '\r' | '\t' => 2,
            control if control.is_control() => {
                format!("\\u{{{:04x}}}", control as u32).chars().count()
            }
            _ => 1,
        })
        .sum()
}

// Approval arguments are host-supplied data, so every line separator, tab,
// C0 and C1 control is rendered as visible text. A value cannot spoof a new
// dialog field or a second line.
fn bounded_approval_text(text: &str) -> String {
    if escaped_approval_len(text) <= MAX_APPROVAL_PREVIEW_CHARS {
        let mut escaped = String::with_capacity(text.len());
        for character in text.chars() {
            escaped.push_str(&escape_approval_character(character));
        }
        return escaped;
    }
    let mut bounded = String::new();
    let mut count = 0usize;
    for character in text.chars() {
        let piece = escape_approval_character(character);
        let length = piece.chars().count();
        if count + length > MAX_APPROVAL_PREVIEW_CHARS.saturating_sub(1) {
            break;
        }
        bounded.push_str(&piece);
        count += length;
    }
    bounded.push('…');
    bounded
}

/// Host-derived preview for the native approval dialog. The page can never
/// supply this text; [`ARGUMENT_AUTHORITY_FIELDS`] refuses that argument.
fn approval_preview(arguments: &Value) -> String {
    const FIELDS: [&str; 9] = [
        "path",
        "cwd",
        "command",
        "query",
        "server",
        "tool",
        "artifact_id",
        "url",
        "content",
    ];
    let Some(object) = arguments.as_object() else {
        return String::new();
    };
    let mut parts = Vec::new();
    for field in FIELDS {
        let Some(value) = object.get(field).and_then(Value::as_str) else {
            continue;
        };
        parts.push(format!("{field}: {}", bounded_approval_text(value)));
    }
    if parts.is_empty() {
        return match canonical_json(arguments) {
            Ok(rendered) => bounded_approval_text(&rendered),
            Err(_) => "(arguments are not canonical)".to_string(),
        };
    }
    bounded_approval_text(&parts.join("\n"))
}

#[cfg(unix)]
fn fill_entropy_from_path(path: &Path, key: &mut [u8]) -> Result<(), String> {
    use std::io::Read;
    let mut file = fs::File::open(path)
        .map_err(|error| format!("the platform CSPRNG is unavailable: {error}"))?;
    file.read_exact(key)
        .map_err(|error| format!("the platform CSPRNG returned an incomplete key: {error}"))
}

#[cfg(unix)]
fn fill_platform_entropy(key: &mut [u8]) -> Result<(), String> {
    fill_entropy_from_path(Path::new("/dev/urandom"), key)
}

#[cfg(not(unix))]
fn fill_platform_entropy(_key: &mut [u8]) -> Result<(), String> {
    Err("no platform CSPRNG is available for the host receipt key on this target".to_string())
}

fn generate_receipt_key() -> Result<[u8; 32], String> {
    let mut key = [0u8; 32];
    fill_platform_entropy(&mut key)?;
    Ok(key)
}

fn is_approval_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn is_digest(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn parse_intent(value: &Value) -> Result<ParsedIntent, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "invalid_arguments: host tool intent must be an object".to_string())?;
    for field in AUTHORITY_FIELDS {
        if object.contains_key(field) {
            return Err(format!(
                "invalid_arguments: host tool intent may not carry authority field {field}"
            ));
        }
    }
    if object.get("schema").and_then(Value::as_str) != Some(HOST_INTENT_SCHEMA) {
        return Err("invalid_arguments: host tool intent schema is not host-intent/1".to_string());
    }
    let string_field = |name: &str, max: usize| -> Result<String, String> {
        let value = object
            .get(name)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("invalid_arguments: host tool intent requires {name}"))?;
        if value.is_empty() || value.len() > max {
            return Err(format!(
                "invalid_arguments: host tool intent {name} must be a bounded non-empty string"
            ));
        }
        Ok(value.to_string())
    };
    let intent_id = string_field("intent_id", 256)?;
    let task_id = string_field("task_id", 64)?;
    let run_id = string_field("run_id", 64)?;
    let tool_call_id = string_field("tool_call_id", 256)?;
    let tool = string_field("tool", 128)?;
    if !tool
        .bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_lowercase())
        || !tool.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'-')
        })
    {
        return Err("invalid_arguments: host tool name is not canonical".to_string());
    }
    let generation = object
        .get("generation")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            "invalid_arguments: host tool intent generation must be a non-negative integer"
                .to_string()
        })?;
    let request_scope = object
        .get("request_scope")
        .and_then(Value::as_str)
        .unwrap_or("task")
        .to_string();
    if request_scope != "task" && request_scope != "data" {
        return Err("invalid_arguments: request_scope must be task | data".to_string());
    }
    if !is_valid_id(&task_id, "tsk") || !is_valid_id(&run_id, "run") {
        return Err(
            "invalid_arguments: host tool intent requires canonical task and run identities"
                .to_string(),
        );
    }
    if request_scope == "data" {
        if task_id != DATA_TASK_ID || run_id != DATA_RUN_ID || generation != 1 {
            return Err(
                "invalid_arguments: host data intent must not claim a real task, run or generation"
                    .to_string(),
            );
        }
    } else if task_id == DATA_TASK_ID || run_id == DATA_RUN_ID || generation < 1 {
        return Err(
            "invalid_arguments: host task intent requires a real task, run and generation"
                .to_string(),
        );
    }
    let arguments = object
        .get("arguments")
        .cloned()
        .ok_or_else(|| "invalid_arguments: host tool intent requires arguments".to_string())?;
    if !arguments.is_object() {
        return Err("invalid_arguments: host tool arguments must be an object".to_string());
    }
    let canonical = canonical_json(&arguments).map_err(|error| {
        format!("invalid_arguments: host tool arguments are not canonical: {error}")
    })?;
    if canonical.len() > MAX_HOST_ARGUMENT_BYTES {
        return Err("invalid_arguments: host tool arguments exceed the host limit".to_string());
    }
    let arguments_digest = object
        .get("arguments_digest")
        .and_then(Value::as_str)
        .ok_or_else(|| "invalid_arguments: host tool intent requires arguments_digest".to_string())?
        .to_string();
    if !is_digest(&arguments_digest) {
        return Err(
            "invalid_arguments: arguments_digest must be sha256:<64 lowercase hex>".to_string(),
        );
    }
    if sha256_prefixed(canonical.as_bytes()) != arguments_digest {
        return Err(
            "invalid_arguments: arguments_digest does not match the canonical arguments"
                .to_string(),
        );
    }
    let approval_id = match object.get("approval_id") {
        None => None,
        Some(value) => {
            let identity = value.as_str().ok_or_else(|| {
                "invalid_arguments: approval_id must be a host-issued identity".to_string()
            })?;
            if !is_approval_identity(identity) {
                return Err(
                    "invalid_arguments: approval_id is not a bounded host identity".to_string(),
                );
            }
            Some(identity.to_string())
        }
    };
    Ok(ParsedIntent {
        intent_id,
        task_id,
        run_id,
        generation,
        tool_call_id,
        tool,
        arguments,
        arguments_digest,
        request_scope,
        approval_id,
    })
}

fn string_argument<'a>(
    arguments: &'a Value,
    key: &str,
    allow_empty: bool,
) -> Result<&'a str, HostFailure> {
    let value = arguments
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| HostFailure::invalid_arguments(format!("{key} must be a string")))?;
    if !allow_empty && value.is_empty() {
        return Err(HostFailure::invalid_arguments(format!(
            "{key} must not be empty"
        )));
    }
    Ok(value)
}

fn optional_string_argument<'a>(
    arguments: &'a Value,
    key: &str,
) -> Result<Option<&'a str>, HostFailure> {
    match arguments.get(key) {
        None => Ok(None),
        Some(value) => value
            .as_str()
            .map(Some)
            .ok_or_else(|| HostFailure::invalid_arguments(format!("{key} must be a string"))),
    }
}

fn optional_u64_argument(arguments: &Value, key: &str) -> Result<Option<u64>, HostFailure> {
    match arguments.get(key) {
        None => Ok(None),
        Some(value) => value.as_u64().map(Some).ok_or_else(|| {
            HostFailure::invalid_arguments(format!("{key} must be a non-negative integer"))
        }),
    }
}

fn validate_relative_path(path: &str) -> Result<&Path, HostFailure> {
    let requested = Path::new(path);
    if path.is_empty() {
        return Err(HostFailure::invalid_arguments("path must not be empty"));
    }
    if requested.is_absolute()
        || requested
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(HostFailure::host_denied(format!(
            "path is outside the granted folder: {path}"
        )));
    }
    reject_parent_dir_traversal(requested).map_err(backend_failure)?;
    Ok(requested)
}

fn media_type_for_path(path: &str) -> String {
    let extension = Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match extension.as_str() {
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "pdf" => "application/pdf",
        "md" | "markdown" => "text/markdown",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "csv" => "text/csv",
        "xml" => "application/xml",
        _ => "text/plain",
    }
    .to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ApprovalDecision {
    Approved,
    Denied,
}

struct HostApprovalRecord {
    approval_id: String,
    task_id: String,
    run_id: String,
    generation: u64,
    tool_call_id: String,
    tool: String,
    request_scope: String,
    request_digest: String,
    scope: String,
    preview: String,
    requested_at_ms: u64,
    expires_at_ms: u64,
    decision: Option<ApprovalDecision>,
}

struct HostSubscriber {
    id: u64,
    sink: Box<dyn Fn(&HostToolResult) + Send>,
}

enum ApprovalOutcome {
    NeedsRequest {
        approval_id: String,
        expires_at_ms: u64,
        already_approved: bool,
    },
    Approved {
        receipt: String,
    },
}

/// Approved-tool executor with no task/run/event state.
pub struct AgentHost {
    backend: DesktopBackend,
    receipt_key: [u8; 32],
    approvals: BTreeMap<String, HostApprovalRecord>,
    results: VecDeque<HostToolResult>,
    next_result_sequence: u64,
    dropped_through: u64,
    subscribers: Vec<HostSubscriber>,
    next_subscriber_id: u64,
    confirmed_mcp_sessions: BTreeSet<String>,
    approval_ttl_ms: u64,
}

impl fmt::Debug for AgentHost {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AgentHost")
            .field("approvals", &self.approvals.len())
            .field("results", &self.results.len())
            .field("subscribers", &self.subscribers.len())
            .field("confirmed_mcp_sessions", &self.confirmed_mcp_sessions.len())
            .field("receipt_key", &"<redacted>")
            .finish()
    }
}

impl AgentHost {
    /// Wrap the existing desktop backend authority.
    ///
    /// Startup fails when the platform CSPRNG is unavailable: there is no
    /// PID/time fallback for the host receipt key.
    pub fn new(backend: DesktopBackend) -> Result<Self, String> {
        Ok(Self {
            backend,
            receipt_key: generate_receipt_key()?,
            approvals: BTreeMap::new(),
            results: VecDeque::new(),
            next_result_sequence: 1,
            dropped_through: 0,
            subscribers: Vec::new(),
            next_subscriber_id: 1,
            confirmed_mcp_sessions: BTreeSet::new(),
            approval_ttl_ms: HOST_APPROVAL_TTL_MS,
        })
    }

    pub fn backend(&self) -> &DesktopBackend {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut DesktopBackend {
        &mut self.backend
    }

    /// Host-owned policy: how long a request stays decidable/executable.
    pub fn set_approval_ttl_ms(&mut self, ttl_ms: u64) {
        self.approval_ttl_ms = ttl_ms.max(1);
    }

    /// Host-owned: record that a native MCP dialog confirmed `session_id`.
    pub fn register_confirmed_mcp_session(&mut self, session_id: impl Into<String>) {
        self.confirmed_mcp_sessions.insert(session_id.into());
    }

    /// Host-owned: forget a session that was disconnected.
    pub fn unregister_confirmed_mcp_session(&mut self, session_id: &str) {
        self.confirmed_mcp_sessions.remove(session_id);
    }

    /// Capabilities and the read-only grant list for a surface that can open
    /// the host's native confirmation dialog.
    pub fn capabilities(&self) -> Value {
        self.capabilities_for_surface(true)
    }

    /// Capabilities for an explicit confirmation surface.
    ///
    /// A dialog-less entrypoint (the loopback JSON-RPC shim or the offline
    /// stdio runner) must report `approvals: false`: approval-required tools
    /// can only be denied there, never approved by the page.
    pub fn capabilities_for_surface(&self, approvals_available: bool) -> Value {
        json!({
            "tree": true,
            "changes": true,
            "readArtifact": true,
            "terminal": self.backend.terminal_enabled(),
            "git": true,
            "mcp": true,
            "approvals": approvals_available,
            "runtime": false,
            "grants": self.backend.agent_grants(),
        })
    }

    /// Look up one host-held approval request for a native confirmation dialog.
    pub fn pending_approval_summary(
        &self,
        approval_id: &str,
    ) -> Result<HostApprovalSummary, String> {
        let record = self.approvals.get(approval_id).ok_or_else(|| {
            "approval_forbidden: approval identity is not held by this host".to_string()
        })?;
        if now_ms() >= record.expires_at_ms {
            return Err("approval_expired: the approval request has expired".to_string());
        }
        Ok(HostApprovalSummary {
            approval_id: record.approval_id.clone(),
            task_id: record.task_id.clone(),
            run_id: record.run_id.clone(),
            generation: record.generation,
            tool_call_id: record.tool_call_id.clone(),
            tool: record.tool.clone(),
            scope: record.scope.clone(),
            preview: record.preview.clone(),
            request_digest: record.request_digest.clone(),
            requested_at_ms: record.requested_at_ms,
            expires_at_ms: record.expires_at_ms,
            approved: record.decision == Some(ApprovalDecision::Approved),
            resolved: record.decision.is_some(),
        })
    }

    /// Execute one validated intent or return an explicit refusal.
    pub fn invoke(
        &mut self,
        mcp: &dyn HostMcpDispatch,
        intent_value: Value,
    ) -> Result<HostToolResult, String> {
        let intent = parse_intent(&intent_value)?;
        let Some(kind) = ToolKind::for_name(&intent.tool) else {
            let failure = HostFailure::invalid_arguments(format!(
                "the host does not execute tool {}",
                intent.tool
            ));
            return Ok(self.finish(intent, "error", None, Some(failure), None, None));
        };
        if kind.is_data() != (intent.request_scope == "data") {
            return Err(
                "invalid_arguments: host request_scope does not match the tool".to_string(),
            );
        }
        if let Some(arguments) = intent.arguments.as_object() {
            for field in ARGUMENT_AUTHORITY_FIELDS {
                if arguments.contains_key(field) {
                    let failure = HostFailure::invalid_arguments(format!(
                        "tool arguments may not carry authority field {field}"
                    ));
                    return Ok(self.finish(intent, "error", None, Some(failure), None, None));
                }
            }
        }

        if let Err(failure) = self.prevalidate(&intent, kind) {
            return if intent.approval_id.is_some() {
                Err(format!("{}: {}", failure.code, failure.message))
            } else {
                Ok(self.finish(intent, "error", None, Some(failure), None, None))
            };
        }

        let mut receipt = None;
        if kind.needs_approval() {
            match self.authorize_approval(&intent)? {
                ApprovalOutcome::NeedsRequest {
                    approval_id,
                    expires_at_ms,
                    already_approved,
                } => {
                    let failure = HostFailure::approval_forbidden(
                        "host approval is required; approve it in the local companion",
                    );
                    let output = json!({
                        "approval_required": true,
                        "approval_id": approval_id,
                        "expires_at_ms": expires_at_ms,
                        "approved": already_approved,
                    });
                    return Ok(self.finish(
                        intent,
                        "error",
                        Some(output),
                        Some(failure),
                        None,
                        None,
                    ));
                }
                ApprovalOutcome::Approved { receipt: value } => receipt = Some(value),
            }
        } else if intent.approval_id.is_some() {
            return Err(
                "approval_forbidden: this tool does not accept an approval identity".to_string(),
            );
        }

        match self.execute(&intent, kind, mcp) {
            Ok((output, artifact)) => {
                Ok(self.finish(intent, "ok", Some(output), None, artifact, receipt))
            }
            Err(failure) => Ok(self.finish(intent, "error", None, Some(failure), None, receipt)),
        }
    }

    /// Validate an intent without executing it, so an HTTP transport can
    /// reject malformed bodies before it acknowledges the request.
    pub fn validate_intent(value: &Value) -> Result<(), String> {
        parse_intent(value).map(|_| ())
    }

    /// Execute one validated intent and always return a transport result.
    ///
    /// `invoke` reports parse/ownership/approval failures as `Err`; the HTTP
    /// transport needs a canonical `HostToolResult` for the browser instead.
    /// A failure after parsing therefore becomes an explicit error result, not
    /// a dropped request.
    pub fn invoke_transport(
        &mut self,
        mcp: &dyn HostMcpDispatch,
        intent_value: Value,
    ) -> Result<HostToolResult, String> {
        let parsed = parse_intent(&intent_value)?;
        match self.invoke(mcp, intent_value) {
            Ok(result) => Ok(result),
            Err(error) => {
                let code = error_code(&error);
                let message = error
                    .split_once(": ")
                    .map(|(_, message)| message.to_string())
                    .unwrap_or_else(|| error.clone());
                Ok(self.finish(
                    parsed,
                    "error",
                    None,
                    Some(HostFailure::new(code, message)),
                    None,
                    None,
                ))
            }
        }
    }

    /// Return transport results after `cursor`, plus the latest result cursor.
    pub fn poll_results(&self, cursor: Option<&str>) -> Result<(u64, Vec<HostToolResult>), String> {
        let after = match cursor {
            None => 0,
            Some(value) => value.parse::<u64>().map_err(|_| {
                "invalid_arguments: cursor must be a non-negative integer".to_string()
            })?,
        };
        if after < self.dropped_through {
            return Err(
                "cursor_expired: the host no longer retains this result cursor".to_string(),
            );
        }
        let latest = self.next_result_sequence.saturating_sub(1);
        if after > latest {
            return Err("out_of_order: cursor is ahead of the host result sequence".to_string());
        }
        let results: Vec<HostToolResult> = self
            .results
            .iter()
            .filter(|result| result_sequence(result) > after)
            .cloned()
            .collect();
        Ok((latest, results))
    }

    /// Return transport results after `cursor`, plus the latest result cursor.
    pub fn poll(&self, cursor: Option<&str>) -> Result<Value, String> {
        let (latest, results) = self.poll_results(cursor)?;
        Ok(json!({ "cursor": latest.to_string(), "results": results }))
    }

    /// Register one Channel/poll sink. Old sinks are evicted at the bound.
    pub fn subscribe(
        &mut self,
        sink: Box<dyn Fn(&HostToolResult) + Send>,
    ) -> Result<Value, String> {
        self.subscribe_with_id(sink).map(|(_, value)| value)
    }

    /// Register one sink and return its identity so a finite stream can release
    /// itself without waiting for the eviction bound.
    pub fn subscribe_with_id(
        &mut self,
        sink: Box<dyn Fn(&HostToolResult) + Send>,
    ) -> Result<(u64, Value), String> {
        if self.subscribers.len() >= MAX_HOST_SUBSCRIBERS {
            self.subscribers.remove(0);
        }
        let id = self.next_subscriber_id;
        self.next_subscriber_id += 1;
        self.subscribers.push(HostSubscriber { id, sink });
        Ok((id, json!({ "subscribed": true, "subscription_id": id })))
    }

    /// Release one sink when its transport ends.
    pub fn unsubscribe(&mut self, subscription_id: u64) {
        self.subscribers
            .retain(|subscriber| subscriber.id != subscription_id);
    }

    /// Resolve a host-issued approval by page decision.
    ///
    /// `native_confirmed` is the answer from the host's own dialog/authority. A
    /// page `approved` decision without it is refused.
    pub fn resolve_approval(
        &mut self,
        input: &Value,
        native_confirmed: bool,
    ) -> Result<Value, String> {
        let object = input
            .as_object()
            .ok_or_else(|| "invalid_arguments: approval input must be an object".to_string())?;
        if object.len() != 2
            || !object.contains_key("approval_id")
            || !object.contains_key("decision")
        {
            return Err(
                "approval_forbidden: approval input accepts only approval_id and decision"
                    .to_string(),
            );
        }
        let approval_id = object["approval_id"]
            .as_str()
            .filter(|value| is_approval_identity(value))
            .ok_or_else(|| "invalid_arguments: approval_id is not a host identity".to_string())?;
        let decision = object["decision"]
            .as_str()
            .ok_or_else(|| "invalid_arguments: decision must be approved | denied".to_string())?;
        if decision != "approved" && decision != "denied" {
            return Err("invalid_arguments: decision must be approved | denied".to_string());
        }
        let record = self.approvals.get(approval_id).ok_or_else(|| {
            "approval_forbidden: approval identity was not issued by this host".to_string()
        })?;
        if record.decision.is_some() {
            return Err("approval_forbidden: approval was already resolved".to_string());
        }
        if now_ms() >= record.expires_at_ms {
            self.approvals.remove(approval_id);
            return Err("approval_expired: the approval request has expired".to_string());
        }
        if decision == "approved" && !native_confirmed {
            return Err(
                "approval_forbidden: host confirmation is required to approve an intent"
                    .to_string(),
            );
        }
        let resolved = if decision == "approved" {
            ApprovalDecision::Approved
        } else {
            ApprovalDecision::Denied
        };
        let expires_at_ms = record.expires_at_ms;
        if let Some(record) = self.approvals.get_mut(approval_id) {
            record.decision = Some(resolved);
        }
        Ok(json!({
            "approval_id": approval_id,
            "decision": decision,
            "expires_at_ms": expires_at_ms,
        }))
    }

    /// Whether `receipt` is the host receipt for this exact intent identity.
    pub fn receipt_is_valid_for(&self, intent: &Value, receipt: &str) -> bool {
        let Ok(parsed) = parse_intent(intent) else {
            return false;
        };
        if parsed.approval_id.is_none() {
            return false;
        }
        constant_time_eq(&self.receipt_for(&parsed), receipt)
    }

    fn prune_expired_approvals(&mut self) {
        let now = now_ms();
        self.approvals
            .retain(|_, record| now < record.expires_at_ms);
    }

    fn mint_approval_id(&self) -> Result<String, String> {
        for _ in 0..8 {
            let mut bytes = [0u8; 8];
            fill_platform_entropy(&mut bytes)?;
            let candidate = format!("apr_{}", hex::encode(bytes));
            if !self.approvals.contains_key(&candidate) {
                return Ok(candidate);
            }
        }
        Err("internal: the host could not mint a unique approval identity".to_string())
    }

    fn find_binding(&self, intent: &ParsedIntent) -> Option<(String, u64, bool)> {
        let now = now_ms();
        self.approvals
            .values()
            .find(|record| {
                record.task_id == intent.task_id
                    && record.run_id == intent.run_id
                    && record.generation == intent.generation
                    && record.tool_call_id == intent.tool_call_id
                    && record.tool == intent.tool
                    && record.request_scope == intent.request_scope
                    && record.request_digest == intent.arguments_digest
                    && now < record.expires_at_ms
            })
            .map(|record| {
                (
                    record.approval_id.clone(),
                    record.expires_at_ms,
                    record.decision == Some(ApprovalDecision::Approved),
                )
            })
    }

    fn authorize_approval(&mut self, intent: &ParsedIntent) -> Result<ApprovalOutcome, String> {
        let Some(approval_id) = intent.approval_id.as_deref() else {
            if let Some((approval_id, expires_at_ms, already_approved)) = self.find_binding(intent)
            {
                return Ok(ApprovalOutcome::NeedsRequest {
                    approval_id,
                    expires_at_ms,
                    already_approved,
                });
            }
            self.prune_expired_approvals();
            if self.approvals.len() >= MAX_PENDING_APPROVALS {
                return Err(
                    "limit_exceeded: too many pending host approvals; resolve or expire one"
                        .to_string(),
                );
            }
            let approval_id = self.mint_approval_id()?;
            let requested_at_ms = now_ms();
            let expires_at_ms = requested_at_ms.saturating_add(self.approval_ttl_ms);
            self.approvals.insert(
                approval_id.clone(),
                HostApprovalRecord {
                    approval_id: approval_id.clone(),
                    task_id: intent.task_id.clone(),
                    run_id: intent.run_id.clone(),
                    generation: intent.generation,
                    tool_call_id: intent.tool_call_id.clone(),
                    tool: intent.tool.clone(),
                    request_scope: intent.request_scope.clone(),
                    request_digest: intent.arguments_digest.clone(),
                    scope: format!("tool:{}", intent.tool),
                    preview: approval_preview(&intent.arguments),
                    requested_at_ms,
                    expires_at_ms,
                    decision: None,
                },
            );
            return Ok(ApprovalOutcome::NeedsRequest {
                approval_id,
                expires_at_ms,
                already_approved: false,
            });
        };

        let Some(record) = self.approvals.get(approval_id) else {
            return Err(
                "approval_forbidden: approval identity was not issued by this host".to_string(),
            );
        };
        check_binding(record, intent)?;
        let expires_at_ms = record.expires_at_ms;
        let decision = record.decision;
        if now_ms() >= expires_at_ms {
            self.approvals.remove(approval_id);
            return Err("approval_expired: the approval request has expired".to_string());
        }
        match decision {
            None => Err("approval_forbidden: the approval request is still pending".to_string()),
            Some(ApprovalDecision::Denied) => {
                Err("approval_forbidden: the approval request was denied".to_string())
            }
            Some(ApprovalDecision::Approved) => {
                let receipt = self.receipt_for(intent);
                self.approvals.remove(approval_id);
                Ok(ApprovalOutcome::Approved { receipt })
            }
        }
    }

    fn receipt_for(&self, intent: &ParsedIntent) -> String {
        let message = format!(
            "{HOST_RECEIPT_DOMAIN}{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            intent.intent_id,
            intent.task_id,
            intent.run_id,
            intent.generation,
            intent.tool_call_id,
            intent.request_scope,
            intent.arguments_digest,
            intent.approval_id.as_deref().unwrap_or(""),
        );
        format!(
            "hmac-sha256:{}",
            hmac_sha256_hex(&self.receipt_key, message.as_bytes())
        )
    }

    fn resolve_grant(&self, arguments: &Value) -> Result<(String, PathBuf), HostFailure> {
        let Some(grant_id) = optional_string_argument(arguments, "grant_id")? else {
            let grants = self.backend.agent_grants();
            return match grants.as_slice() {
                [] => Err(HostFailure::host_denied(
                    "no host folder grant is available",
                )),
                [grant] => {
                    let path = self
                        .backend
                        .agent_grant_path(&grant.grant_id)
                        .map_err(backend_failure)?;
                    Ok((grant.grant_id.clone(), path))
                }
                _ => Err(HostFailure::invalid_arguments(
                    "grant_id is required when the host holds more than one grant",
                )),
            };
        };
        let path = self
            .backend
            .agent_grant_path(grant_id)
            .map_err(backend_failure)?;
        Ok((grant_id.to_string(), path))
    }

    fn prevalidate(&self, intent: &ParsedIntent, kind: ToolKind) -> Result<(), HostFailure> {
        let arguments = &intent.arguments;
        match kind {
            ToolKind::Tree | ToolKind::Changes | ToolKind::GitStatus => {
                if let Some(scope) = optional_string_argument(arguments, "scope")? {
                    if kind == ToolKind::Changes && scope != "branch" {
                        return Err(HostFailure::invalid_arguments(
                            "git.changes is branch-only; last-turn changes are browser-owned",
                        ));
                    }
                }
                self.resolve_grant(arguments)?;
            }
            ToolKind::ArtifactRead => {
                string_argument(arguments, "artifact_id", false)?;
                let start = optional_u64_argument(arguments, "start")?;
                let end = optional_u64_argument(arguments, "end")?;
                if start.is_some() != end.is_some() {
                    return Err(HostFailure::invalid_arguments(
                        "artifact range requires both start and end",
                    ));
                }
                if let (Some(start), Some(end)) = (start, end) {
                    if start > end {
                        return Err(HostFailure::invalid_arguments(
                            "artifact range start must not exceed end",
                        ));
                    }
                }
                optional_string_argument(arguments, "expected_digest")?;
                optional_u64_argument(arguments, "expected_size_bytes")?;
            }
            ToolKind::FileRead | ToolKind::FileWrite => {
                let (_, root) = self.resolve_grant(arguments)?;
                let path = string_argument(arguments, "path", false)?;
                let relative = validate_relative_path(path)?;
                let target = root.join(relative);
                if kind == ToolKind::FileWrite {
                    if let Some(media_type) = optional_string_argument(arguments, "media_type")? {
                        let _ = media_type;
                    }
                    if let Some(content) = optional_string_argument(arguments, "content")? {
                        if content.len() > MAX_WRITE_FILE_BYTES {
                            return Err(HostFailure::new(
                                "limit_exceeded",
                                format!(
                                    "file content exceeds the {} byte write limit",
                                    MAX_WRITE_FILE_BYTES
                                ),
                            ));
                        }
                    }
                    self.backend
                        .verify_granted_write_path(&target)
                        .map_err(backend_failure)?;
                } else {
                    self.backend
                        .verify_granted_existing_path(&target)
                        .map_err(backend_failure)?;
                }
            }
            ToolKind::TerminalRun => {
                let cwd = string_argument(arguments, "cwd", false)?;
                let command = string_argument(arguments, "command", false)?;
                if !self.backend.terminal_enabled() {
                    return Err(HostFailure::host_denied(
                        "TerminalDisabledByDefault: terminal execution is disabled by the host",
                    ));
                }
                if !self
                    .backend
                    .host_terminal_command_is_approved(Path::new(cwd), command)
                {
                    return Err(HostFailure::host_denied(
                        "TerminalConsentRequired: this exact command has no host approval",
                    ));
                }
            }
            ToolKind::TerminalOpen => {
                string_argument(arguments, "cwd", false)?;
                optional_string_argument(arguments, "shell")?;
            }
            ToolKind::TerminalInput => {
                string_argument(arguments, "session_id", false)?;
                string_argument(arguments, "data", true)?;
            }
            ToolKind::McpCall => {
                let session_id = string_argument(arguments, "session_id", false)?;
                if !self.confirmed_mcp_sessions.contains(session_id) {
                    return Err(HostFailure::host_denied(
                        "host_denied: this MCP session has no host confirmation",
                    ));
                }
                string_argument(arguments, "tool", false)?;
                if !arguments.get("arguments").is_some_and(Value::is_object) {
                    return Err(HostFailure::invalid_arguments(
                        "MCP tool arguments must be an object",
                    ));
                }
            }
        }
        Ok(())
    }

    fn execute(
        &mut self,
        intent: &ParsedIntent,
        kind: ToolKind,
        mcp: &dyn HostMcpDispatch,
    ) -> Result<(Value, Option<HostArtifactRef>), HostFailure> {
        let arguments = &intent.arguments;
        match kind {
            ToolKind::Tree => {
                let (grant_id, _root) = self.resolve_grant(arguments)?;
                let path = optional_string_argument(arguments, "path")?.unwrap_or("");
                let page = self
                    .backend
                    .agent_list_directory(&grant_id, path)
                    .map_err(backend_failure)?;
                let entries: Vec<Value> = page
                    .entries
                    .iter()
                    .map(|entry| {
                        let mut value = json!({
                            "path": entry.path,
                            "name": entry.name,
                            "kind": entry.kind,
                        });
                        if let Some(size_bytes) = entry.size_bytes {
                            value["size_bytes"] = json!(size_bytes);
                        }
                        value
                    })
                    .collect();
                Ok((
                    json!({
                        "grant_id": page.grant_id,
                        "path": page.path,
                        "entries": entries,
                        "truncated": page.truncated,
                        "symlinks_skipped": page.symlinks_skipped,
                        "read_errors": page.read_errors,
                    }),
                    None,
                ))
            }
            ToolKind::Changes => {
                let (grant_id, _root) = self.resolve_grant(arguments)?;
                let scope = optional_string_argument(arguments, "scope")?.unwrap_or("branch");
                let value =
                    crate::agent_data::agent_git_diff_output(&self.backend, &grant_id, scope)
                        .map_err(backend_failure)?;
                Ok((value, None))
            }
            ToolKind::GitStatus => {
                let (_grant_id, root) = self.resolve_grant(arguments)?;
                let report = self
                    .backend
                    .native_git_status(&root)
                    .map_err(backend_failure)?;
                let value = serde_json::to_value(report).map_err(|error| {
                    internal_failure(format!("Git status serialization failed: {error}"))
                })?;
                Ok((value, None))
            }
            ToolKind::ArtifactRead => {
                let artifact_id = string_argument(arguments, "artifact_id", false)?;
                let start = optional_u64_argument(arguments, "start")?;
                let end = optional_u64_argument(arguments, "end")?;
                let range = match (start, end) {
                    (None, None) => None,
                    (Some(start), Some(end)) => Some(AgentArtifactRange { start, end }),
                    _ => {
                        return Err(HostFailure::invalid_arguments(
                            "artifact range requires both start and end",
                        ))
                    }
                };
                let expected_digest = optional_string_argument(arguments, "expected_digest")?;
                let expected_size = optional_u64_argument(arguments, "expected_size_bytes")?;
                let bytes = self
                    .backend
                    .agent_read_artifact(artifact_id, range, expected_digest, expected_size)
                    .map_err(backend_failure)?;
                let value = serde_json::to_value(bytes).map_err(|error| {
                    internal_failure(format!("artifact serialization failed: {error}"))
                })?;
                Ok((value, None))
            }
            ToolKind::FileRead => {
                let (grant_id, _root) = self.resolve_grant(arguments)?;
                let path = string_argument(arguments, "path", false)?;
                let media_type = optional_string_argument(arguments, "media_type")?
                    .map(str::to_string)
                    .unwrap_or_else(|| media_type_for_path(path));
                let info = self
                    .backend
                    .agent_register_artifact(&grant_id, path, &media_type)
                    .map_err(backend_failure)?;
                let artifact = HostArtifactRef {
                    artifact_id: info.artifact_id.clone(),
                    digest: info.digest.clone(),
                    media_type: info.media_type.clone(),
                    size_bytes: info.size_bytes,
                };
                Ok((
                    json!({
                        "path": info.path,
                        "media_type": info.media_type,
                        "size_bytes": info.size_bytes,
                        "digest": info.digest,
                    }),
                    Some(artifact),
                ))
            }
            ToolKind::FileWrite => {
                let (grant_id, root) = self.resolve_grant(arguments)?;
                let path = string_argument(arguments, "path", false)?;
                let content = string_argument(arguments, "content", true)?;
                let relative = validate_relative_path(path)?;
                let target = root.join(relative);
                self.backend
                    .native_write_file(&target, content)
                    .map_err(backend_failure)?;
                let media_type = optional_string_argument(arguments, "media_type")?
                    .map(str::to_string)
                    .unwrap_or_else(|| media_type_for_path(path));
                let size_bytes = content.len() as u64;
                let digest = sha256_prefixed(content.as_bytes());
                let mut artifact = None;
                let regular_file = fs::metadata(&target)
                    .map(|metadata| metadata.is_file())
                    .unwrap_or(false);
                if regular_file && size_bytes <= MAX_AGENT_ARTIFACT_BYTES {
                    if let Ok(info) =
                        self.backend
                            .agent_register_artifact(&grant_id, path, &media_type)
                    {
                        artifact = Some(HostArtifactRef {
                            artifact_id: info.artifact_id,
                            digest: info.digest,
                            media_type: info.media_type,
                            size_bytes: info.size_bytes,
                        });
                    }
                }
                Ok((
                    json!({
                        "path": path,
                        "media_type": media_type,
                        "size_bytes": size_bytes,
                        "digest": digest,
                    }),
                    artifact,
                ))
            }
            ToolKind::TerminalRun => {
                let cwd = string_argument(arguments, "cwd", false)?;
                let command = string_argument(arguments, "command", false)?;
                let request = crate::TerminalCommandRequest {
                    cwd: PathBuf::from(cwd),
                    command: command.to_string(),
                    user_confirmed: true,
                    confirmed_command: command.to_string(),
                };
                let output = self
                    .backend
                    .native_run_terminal_command(&request)
                    .map_err(backend_failure)?;
                Ok((json!({ "output": output }), None))
            }
            ToolKind::TerminalOpen => {
                let cwd = string_argument(arguments, "cwd", false)?;
                let shell = optional_string_argument(arguments, "shell")?.map(Path::new);
                let rows = optional_u64_argument(arguments, "rows")?
                    .unwrap_or(24)
                    .try_into()
                    .map_err(|_| HostFailure::invalid_arguments("rows is too large"))?;
                let cols = optional_u64_argument(arguments, "cols")?
                    .unwrap_or(80)
                    .try_into()
                    .map_err(|_| HostFailure::invalid_arguments("cols is too large"))?;
                let snapshot = self
                    .backend
                    .open_terminal(Path::new(cwd), shell, rows, cols)
                    .map_err(backend_failure)?;
                let value = serde_json::to_value(snapshot).map_err(|error| {
                    internal_failure(format!("terminal snapshot serialization failed: {error}"))
                })?;
                Ok((value, None))
            }
            ToolKind::TerminalInput => {
                let session_id = string_argument(arguments, "session_id", false)?;
                let data = string_argument(arguments, "data", true)?;
                self.backend
                    .write_terminal(session_id, data.as_bytes())
                    .map_err(backend_failure)?;
                Ok((json!({ "ok": true }), None))
            }
            ToolKind::McpCall => {
                let session_id = string_argument(arguments, "session_id", false)?;
                let tool = string_argument(arguments, "tool", false)?;
                let mcp_arguments = arguments
                    .get("arguments")
                    .cloned()
                    .ok_or_else(|| HostFailure::invalid_arguments("MCP arguments are required"))?;
                let value = mcp
                    .call_tool(session_id, tool, &mcp_arguments)
                    .map_err(|error| {
                        HostFailure::new("internal", format!("MCP tool call failed: {error}"))
                    })?;
                Ok((value, None))
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn finish(
        &mut self,
        intent: ParsedIntent,
        status: &str,
        output: Option<Value>,
        failure: Option<HostFailure>,
        artifact: Option<HostArtifactRef>,
        receipt: Option<String>,
    ) -> HostToolResult {
        let (output, canonical, overridden) = match output {
            None => (None, "{}".to_string(), None),
            Some(value) => match canonical_json(&value) {
                Ok(canonical) => (Some(value), canonical, None),
                Err(error) => (
                    None,
                    "{}".to_string(),
                    Some(HostFailure::new(
                        "invalid_event",
                        format!("host result output is not canonical: {error}"),
                    )),
                ),
            },
        };
        let (status, failure) = match overridden {
            Some(error) => ("error", Some(error)),
            None => (status, failure),
        };
        let mut result = HostToolResult {
            schema: HOST_RESULT_SCHEMA.to_string(),
            intent_id: intent.intent_id,
            task_id: intent.task_id,
            run_id: intent.run_id,
            generation: intent.generation,
            tool_call_id: intent.tool_call_id,
            request_scope: Some(intent.request_scope),
            status: status.to_string(),
            output,
            output_digest: sha256_prefixed(canonical.as_bytes()),
            output_preview: preview(&canonical),
            artifact,
            approval_receipt: receipt,
            occurred_at_ms: now_ms(),
            code: failure.as_ref().map(|failure| failure.code.to_string()),
            message: failure.map(|failure| failure.message),
            truncated: None,
            cursor: None,
        };
        let sequence = self.next_result_sequence;
        self.next_result_sequence += 1;
        result.cursor = Some(sequence.to_string());
        self.results.push_back(result.clone());
        while self.results.len() > MAX_HOST_RESULTS {
            if let Some(dropped) = self.results.pop_front() {
                self.dropped_through = result_sequence(&dropped);
            }
        }
        for subscriber in &self.subscribers {
            let _ = catch_unwind(AssertUnwindSafe(|| (subscriber.sink)(&result)));
        }
        result
    }
}

fn result_sequence(result: &HostToolResult) -> u64 {
    result
        .cursor
        .as_deref()
        .and_then(|cursor| cursor.parse::<u64>().ok())
        .unwrap_or(0)
}

fn check_binding(record: &HostApprovalRecord, intent: &ParsedIntent) -> Result<(), String> {
    if record.task_id != intent.task_id {
        return Err("cross_task_event: the approval belongs to a different task".to_string());
    }
    if record.run_id != intent.run_id {
        return Err("out_of_order: the approval belongs to a different run".to_string());
    }
    if record.generation != intent.generation {
        return Err("stale_generation: the approval belongs to a stale generation".to_string());
    }
    if record.tool_call_id != intent.tool_call_id {
        return Err("out_of_order: the approval belongs to a different tool call".to_string());
    }
    if record.tool != intent.tool || record.request_scope != intent.request_scope {
        return Err("approval_forbidden: the approval does not cover this tool".to_string());
    }
    if record.request_digest != intent.arguments_digest {
        return Err(
            "approval_digest_mismatch: the approval does not cover these arguments".to_string(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod approval_preview_and_entropy_tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn missing_platform_entropy_is_an_error_not_a_weak_key() {
        let mut key = [0u8; 32];
        let error = fill_entropy_from_path(Path::new("/nonexistent/uor-r4-entropy"), &mut key)
            .expect_err("a missing CSPRNG source must fail host startup");
        assert!(error.contains("CSPRNG"), "{error}");
        assert_eq!(key, [0u8; 32]);
    }

    #[test]
    fn approval_preview_is_bounded_and_utf8_safe() {
        let preview = approval_preview(&json!({
            "content": format!("{}😀", "x".repeat(400)),
        }));
        assert!(preview.chars().count() <= MAX_APPROVAL_PREVIEW_CHARS);
        assert!(preview.contains("content: "));
    }

    #[test]
    fn approval_preview_escapes_line_and_control_characters() {
        let preview = approval_preview(&json!({
            "command": "safe\r\npath: /etc/passwd\t\u{7}\u{85}\u{7f}",
        }));
        assert!(
            preview.contains("command: safe\\r\\npath: /etc/passwd\\t\\u{0007}\\u{0085}\\u{007f}"),
            "controls must be escaped as visible text: {preview:?}"
        );
        assert!(!preview.contains('\r'));
        assert!(!preview.contains('\t'));
        assert_eq!(
            preview.chars().filter(|character| *character == '\n').count(),
            0,
            "a field value must not introduce an approval-preview line"
        );
        assert!(!preview.chars().any(char::is_control));
    }

    #[test]
    fn approval_preview_cannot_spoof_a_second_field() {
        let preview = approval_preview(&json!({
            "path": "notes.md\ncommand: rm -rf /",
        }));
        assert_eq!(preview, "path: notes.md\\ncommand: rm -rf /");
        assert!(!preview.contains('\n'));
    }
}
