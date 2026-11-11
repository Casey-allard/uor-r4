//! Host-authority tests for approved Studio tool intents.
//!
//! The page owns task/run/event identity; this host only validates an intent,
//! resolves its own approval identity, executes an approved tool and reports a
//! HostToolResult. It never owns task state, an event log or a task cursor.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};
use uor_r4_desktop_client::agent_host::{
    approval_dialog_message, hmac_sha256_hex, AgentHost, HostMcpDispatch, HostToolResult,
    HOST_INTENT_SCHEMA, HOST_RESULT_SCHEMA, MAX_APPROVAL_PREVIEW_CHARS,
};
use uor_r4_desktop_client::{handle_rpc_line_with_host, DesktopBackend, PinRegistry};

const TASK_A: &str = "tsk_9a3c5e7b1d2f4a60";
const RUN_A: &str = "run_7f3a2c1b9d4e6a80";
const TASK_B: &str = "tsk_1122334455667788";
const RUN_B: &str = "run_8877665544332211";
const DATA_TASK: &str = "tsk_0000000000000000";
const DATA_RUN: &str = "run_0000000000000000";

static INTENT_SEQUENCE: AtomicU64 = AtomicU64::new(1);
static TOOL_CALL_SEQUENCE: AtomicU64 = AtomicU64::new(1);

fn unique_temp_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "uor-agent-authority-{label}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn digest_of(value: &Value) -> String {
    let canonical = uor_r4_studio_agent::canonical_json(value).expect("canonical fixture");
    format!(
        "sha256:{}",
        uor_r4_desktop_client::sha256_bytes(canonical.as_bytes())
    )
}

fn host_with_grant(root: &Path) -> (AgentHost, String) {
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend
        .native_grant_folder(root)
        .expect("grant fixture folder");
    let grant_id = backend.agent_grant_id(root).expect("derive grant id");
    (AgentHost::new(backend).expect("secure host"), grant_id)
}

#[allow(clippy::too_many_arguments)]
fn task_intent_raw(
    tool: &str,
    arguments: Value,
    arguments_digest: String,
    approval_id: Option<&str>,
    task_id: &str,
    run_id: &str,
    generation: u64,
    tool_call_id: &str,
) -> Value {
    let intent_id = format!(
        "hti_test_{:012}",
        INTENT_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    );
    let mut intent = json!({
        "schema": HOST_INTENT_SCHEMA,
        "intent_id": intent_id,
        "task_id": task_id,
        "run_id": run_id,
        "generation": generation,
        "tool_call_id": tool_call_id,
        "tool": tool,
        "arguments": arguments,
        "arguments_digest": arguments_digest,
        "request_scope": "task",
    });
    if let Some(approval_id) = approval_id {
        intent["approval_id"] = json!(approval_id);
    }
    intent
}

fn task_intent(tool: &str, arguments: Value, approval_id: Option<&str>) -> Value {
    let digest = digest_of(&arguments);
    let tool_call = format!("tc_{}", &digest["sha256:".len()..][..16]);
    task_intent_raw(
        tool,
        arguments,
        digest,
        approval_id,
        TASK_A,
        RUN_A,
        3,
        &tool_call,
    )
}

fn data_intent(tool: &str, arguments: Value) -> Value {
    let digest = digest_of(&arguments);
    let tool_call = format!(
        "tc_data_{:012}",
        TOOL_CALL_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    );
    let mut intent = task_intent_raw(
        tool, arguments, digest, None, DATA_TASK, DATA_RUN, 1, &tool_call,
    );
    intent["request_scope"] = json!("data");
    intent
}

fn invoke(host: &mut AgentHost, intent: Value) -> HostToolResult {
    host.invoke(&NoMcp, intent)
        .expect("a valid host intent must produce a structured result")
}

fn invoke_error(host: &mut AgentHost, intent: Value) -> HostToolResult {
    let result = invoke(host, intent);
    assert_eq!(
        result.status, "error",
        "expected an error result, got {result:?}"
    );
    result
}

fn request_approval(host: &mut AgentHost, intent: Value) -> String {
    let result = invoke_error(host, intent);
    assert_eq!(
        result.code.as_deref(),
        Some("approval_forbidden"),
        "unexpected approval-request result: {result:?}"
    );
    assert!(
        result.approval_receipt.is_none(),
        "a pending approval must not carry a receipt"
    );
    let output = result.output.expect("approval request output");
    assert_eq!(output["approval_required"], json!(true));
    let expires_at_ms = output["expires_at_ms"]
        .as_u64()
        .expect("approval expiry is an integer");
    assert!(expires_at_ms > 0);
    output["approval_id"]
        .as_str()
        .expect("host-issued approval id")
        .to_string()
}

fn approve(host: &mut AgentHost, approval_id: &str) {
    host.resolve_approval(
        &json!({"approval_id": approval_id, "decision": "approved"}),
        true,
    )
    .expect("host-confirmed approval");
}

fn write_intent(grant_id: &str, path: &str, content: &str) -> Value {
    task_intent(
        "file.write",
        json!({"grant_id": grant_id, "path": path, "content": content}),
        None,
    )
}

struct FakeMcp {
    confirmed: BTreeSet<String>,
    non_canonical: bool,
}

impl FakeMcp {
    fn with_confirmed(id: &str) -> Self {
        let mut confirmed = BTreeSet::new();
        confirmed.insert(id.to_string());
        Self {
            confirmed,
            non_canonical: false,
        }
    }
}

impl HostMcpDispatch for FakeMcp {
    fn list_tools(&self, session_id: &str) -> Result<Value, String> {
        if !self.confirmed.contains(session_id) {
            return Err("MCP session is unavailable.".to_string());
        }
        Ok(json!([{ "name": "echo", "inputSchema": {"type": "object"} }]))
    }

    fn call_tool(&self, session_id: &str, tool: &str, arguments: &Value) -> Result<Value, String> {
        if !self.confirmed.contains(session_id) {
            return Err("MCP session is unavailable.".to_string());
        }
        if self.non_canonical {
            return Ok(json!({"Value": 1.5}));
        }
        Ok(json!({
            "content": [{ "type": "text", "text": "ok" }],
            "tool": tool,
            "arguments": arguments,
        }))
    }
}

struct NoMcp;

impl HostMcpDispatch for NoMcp {
    fn list_tools(&self, _session_id: &str) -> Result<Value, String> {
        Err("MCP is unavailable in this host".to_string())
    }

    fn call_tool(
        &self,
        _session_id: &str,
        _tool: &str,
        _arguments: &Value,
    ) -> Result<Value, String> {
        Err("MCP is unavailable in this host".to_string())
    }
}

#[test]
fn agent_host_consumes_approval_by_identity() {
    let root = unique_temp_dir("consume");
    let (mut host, grant_id) = host_with_grant(&root);
    let request = write_intent(&grant_id, "note.txt", "first");
    let request_tool_call = request["tool_call_id"].clone();
    let approval_id = request_approval(&mut host, request);

    let summary = host
        .pending_approval_summary(&approval_id)
        .expect("pending approval summary");
    assert_eq!(summary.task_id, TASK_A);
    assert_eq!(summary.run_id, RUN_A);
    assert_eq!(summary.generation, 3);
    assert_eq!(summary.tool_call_id, request_tool_call);
    assert_eq!(summary.tool, "file.write");
    assert_eq!(summary.scope, "tool:file.write");
    assert_eq!(
        summary.request_digest,
        digest_of(&json!({
            "grant_id": grant_id,
            "path": "note.txt",
            "content": "first",
        }))
    );
    assert!(summary.expires_at_ms > summary.requested_at_ms);
    assert!(!summary.approved);
    assert!(!summary.resolved);

    approve(&mut host, &approval_id);
    let execution = task_intent(
        "file.write",
        json!({"grant_id": grant_id, "path": "note.txt", "content": "first"}),
        Some(&approval_id),
    );
    let result = invoke(&mut host, execution);
    assert_eq!(
        result.status, "ok",
        "unexpected execution result: {result:?}"
    );
    assert_eq!(result.code, None);
    assert_eq!(result.tool_call_id, request_tool_call);
    let receipt = result
        .approval_receipt
        .expect("approved execution carries a host receipt");
    assert!(receipt.starts_with("hmac-sha256:"));
    assert_eq!(fs::read_to_string(root.join("note.txt")).unwrap(), "first");

    let replay = task_intent(
        "file.write",
        json!({"grant_id": grant_id, "path": "note.txt", "content": "first"}),
        Some(&approval_id),
    );
    let replay_error = host
        .invoke(&NoMcp, replay)
        .expect_err("a consumed approval must be refused");
    assert!(
        replay_error.contains("approval_forbidden"),
        "unexpected replay error: {replay_error}"
    );
    assert_eq!(fs::read_to_string(root.join("note.txt")).unwrap(), "first");
    assert!(host.pending_approval_summary(&approval_id).is_err());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn host_approval_records_retain_a_bounded_host_computed_preview() {
    let root = unique_temp_dir("approval-preview");
    let (mut host, grant_id) = host_with_grant(&root);
    let content = "preview-body ".repeat(80);
    let request = write_intent(&grant_id, "preview.txt", &content);
    let approval_id = request_approval(&mut host, request);

    let summary = host
        .pending_approval_summary(&approval_id)
        .expect("pending approval summary");
    assert!(
        summary.preview.contains("path: preview.txt"),
        "the host must derive the path preview itself: {:?}",
        summary.preview
    );
    assert!(summary.preview.contains("preview-body"));
    assert!(
        summary.preview.chars().count() <= MAX_APPROVAL_PREVIEW_CHARS,
        "preview must stay bounded: {} chars",
        summary.preview.chars().count()
    );
    let dialog = approval_dialog_message(&summary);
    assert!(
        dialog.contains("Preview:\npath: preview.txt"),
        "the native dialog must show the host-derived preview: {dialog}"
    );
    assert!(!dialog.contains("page-supplied authority"));

    let forged = task_intent(
        "file.write",
        json!({
            "grant_id": grant_id,
            "path": "preview.txt",
            "content": "x",
            "preview": "page-supplied authority",
        }),
        None,
    );
    let refused = invoke_error(&mut host, forged);
    assert_eq!(refused.code.as_deref(), Some("invalid_arguments"));
    assert!(
        !host
            .pending_approval_summary(&approval_id)
            .expect("original approval stays pending")
            .preview
            .contains("page-supplied authority"),
        "a page preview must never become host approval copy"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn host_startup_requires_secure_entropy() {
    let backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    let host = AgentHost::new(backend);
    assert!(
        host.is_ok(),
        "this test host must expose a platform CSPRNG for the host receipt key: {:?}",
        host.err()
    );
}

#[test]
fn agent_host_refuses_page_supplied_grant() {
    let root = unique_temp_dir("page-grant");
    let (mut host, grant_id) = host_with_grant(&root);

    let mut forged = write_intent(&grant_id, "note.txt", "value");
    forged["granted"] = json!(true);
    let error = host
        .invoke(&NoMcp, forged)
        .expect_err("a page-supplied granted flag must be refused");
    assert!(
        error.contains("invalid_arguments"),
        "unexpected granted error: {error}"
    );

    let granted_argument = task_intent(
        "file.write",
        json!({
            "grant_id": grant_id,
            "path": "note.txt",
            "content": "value",
            "granted": true,
        }),
        None,
    );
    let result = invoke_error(&mut host, granted_argument);
    assert_eq!(result.code.as_deref(), Some("invalid_arguments"));
    assert!(!root.join("note.txt").exists());

    let unknown_grant = task_intent(
        "file.write",
        json!({
            "grant_id": "gr_0000000000000000",
            "path": "note.txt",
            "content": "value",
        }),
        None,
    );
    let result = invoke_error(&mut host, unknown_grant);
    assert_eq!(result.code.as_deref(), Some("host_denied"));
    assert!(!root.join("note.txt").exists());

    let rpc = handle_rpc_line_with_host(
        &mut host,
        &json!({
            "id": 1,
            "command": "native_grant_folder",
            "args": {"folderPath": root.display().to_string()},
        })
        .to_string(),
    );
    assert_eq!(rpc["ok"], json!(false));
    assert!(
        rpc["error"].as_str().unwrap().contains("host-owned"),
        "page RPC must not create grants: {rpc}"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn agent_host_refuses_forged_origin() {
    let root = unique_temp_dir("forged-origin");
    let (mut host, grant_id) = host_with_grant(&root);
    let request = write_intent(&grant_id, "note.txt", "value");
    let approval_id = request_approval(&mut host, request);

    let unknown = host.resolve_approval(
        &json!({"approval_id": "apr_0000000000000000", "decision": "approved"}),
        true,
    );
    assert!(
        unknown.unwrap_err().contains("approval_forbidden"),
        "an approval identity the host never issued must be refused"
    );

    let forged_origin = host.resolve_approval(
        &json!({
            "approval_id": approval_id,
            "decision": "approved",
            "origin": "host",
            "resolved_by": "host",
        }),
        true,
    );
    assert!(
        forged_origin.unwrap_err().contains("approval_forbidden"),
        "a page-supplied origin must not be accepted as host authority"
    );

    let mut intent_with_origin = write_intent(&grant_id, "note.txt", "value");
    intent_with_origin["origin"] = json!("host");
    let error = host
        .invoke(&NoMcp, intent_with_origin)
        .expect_err("an intent claiming host origin must be refused");
    assert!(
        error.contains("invalid_arguments"),
        "unexpected intent-origin error: {error}"
    );

    let execution = task_intent(
        "file.write",
        json!({"grant_id": grant_id, "path": "note.txt", "content": "value"}),
        Some(&approval_id),
    );
    let still_pending = host
        .invoke(&NoMcp, execution)
        .expect_err("the forged resolution must not approve the intent");
    assert!(still_pending.contains("approval_forbidden"));
    assert!(!root.join("note.txt").exists());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn agent_host_refuses_cross_task_approval() {
    let root = unique_temp_dir("cross-task");
    let (mut host, grant_id) = host_with_grant(&root);
    let request = write_intent(&grant_id, "note.txt", "value");
    let approval_id = request_approval(&mut host, request);
    approve(&mut host, &approval_id);

    let arguments = json!({"grant_id": grant_id, "path": "note.txt", "content": "value"});
    let cross_task = task_intent_raw(
        "file.write",
        arguments.clone(),
        digest_of(&arguments),
        Some(&approval_id),
        TASK_B,
        RUN_A,
        3,
        "tc_cross_task",
    );
    let error = host
        .invoke(&NoMcp, cross_task)
        .expect_err("a cross-task approval must be refused");
    assert!(error.contains("cross_task_event"), "unexpected: {error}");

    let cross_run = task_intent_raw(
        "file.write",
        arguments.clone(),
        digest_of(&arguments),
        Some(&approval_id),
        TASK_A,
        RUN_B,
        3,
        "tc_cross_run",
    );
    let error = host
        .invoke(&NoMcp, cross_run)
        .expect_err("a cross-run approval must be refused");
    assert!(error.contains("out_of_order"), "unexpected: {error}");

    let stale_generation = task_intent_raw(
        "file.write",
        arguments.clone(),
        digest_of(&arguments),
        Some(&approval_id),
        TASK_A,
        RUN_A,
        2,
        "tc_stale_generation",
    );
    let error = host
        .invoke(&NoMcp, stale_generation)
        .expect_err("a stale-generation approval must be refused");
    assert!(error.contains("stale_generation"), "unexpected: {error}");
    assert!(!root.join("note.txt").exists());

    let _ = fs::remove_dir_all(&root);
}

#[cfg(unix)]
#[test]
fn agent_host_stop_does_not_kill_in_flight_write() {
    let root = unique_temp_dir("in-flight");
    let fifo = root.join("pipe.txt");
    let status = Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .expect("mkfifo fixture");
    assert!(status.success(), "mkfifo must create the in-flight fixture");
    let (host, grant_id) = host_with_grant(&root);
    let host = Arc::new(Mutex::new(host));

    let content = "x".repeat(1024 * 1024);
    let execution = {
        let request = write_intent(&grant_id, "pipe.txt", &content);
        let approval_id = {
            let mut guard = host.lock().unwrap();
            request_approval(&mut guard, request)
        };
        host.lock()
            .unwrap()
            .resolve_approval(
                &json!({"approval_id": approval_id, "decision": "approved"}),
                true,
            )
            .expect("host-confirmed approval");
        task_intent(
            "file.write",
            json!({"grant_id": grant_id, "path": "pipe.txt", "content": content}),
            Some(&approval_id),
        )
    };

    let reader_path = fifo.clone();
    let reader = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(200));
        let mut file = fs::File::open(&reader_path).expect("open FIFO read end");
        std::thread::sleep(Duration::from_millis(400));
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).expect("drain FIFO");
        bytes
    });

    let writer_host = Arc::clone(&host);
    let writer = std::thread::spawn(move || writer_host.lock().unwrap().invoke(&NoMcp, execution));

    std::thread::sleep(Duration::from_millis(80));
    let stop_started = std::time::Instant::now();
    let stop = host.lock().unwrap().invoke(
        &NoMcp,
        task_intent("run.stop", json!({"tool_call_id": "tc_in_flight"}), None),
    );
    let stop_elapsed = stop_started.elapsed();

    let written = reader.join().expect("reader thread");
    let writer_result = writer
        .join()
        .expect("writer thread")
        .expect("in-flight write must return a result");
    assert_eq!(written.len(), content.len());
    assert_eq!(written, content.as_bytes());
    assert_eq!(
        writer_result.status, "ok",
        "an in-flight write must be acknowledged with its real completion: {writer_result:?}"
    );
    assert!(
        writer_result
            .approval_receipt
            .as_deref()
            .is_some_and(|receipt| receipt.starts_with("hmac-sha256:")),
        "the approved write must carry its host receipt"
    );

    let stop = stop.expect("the stop intent must produce an explicit refusal");
    assert_eq!(stop.status, "error");
    assert_eq!(stop.code.as_deref(), Some("invalid_arguments"));
    assert_ne!(stop.status, "cancelled");
    assert!(
        stop_elapsed >= Duration::from_millis(200),
        "the stop intent returned in {stop_elapsed:?}, so it did not wait for the in-flight write"
    );
    assert!(!fs::metadata(&fifo).unwrap().file_type().is_file());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn agent_host_terminal_requires_existing_approval() {
    let root = unique_temp_dir("terminal");
    let (mut host, _grant_id) = host_with_grant(&root);
    let cwd = root.display().to_string();

    let unapproved = task_intent(
        "terminal.run",
        json!({
            "cwd": cwd,
            "command": "echo hello",
            "user_confirmed": true,
            "confirmed_command": "echo hello",
        }),
        None,
    );
    let result = invoke_error(&mut host, unapproved);
    assert_eq!(result.code.as_deref(), Some("host_denied"));
    assert!(
        result
            .message
            .as_deref()
            .unwrap_or("")
            .contains("TerminalDisabledByDefault"),
        "terminal must remain disabled by default: {result:?}"
    );

    host.backend_mut().native_set_terminal_enabled(true);
    let unapproved = task_intent(
        "terminal.run",
        json!({"cwd": cwd, "command": "echo hello"}),
        None,
    );
    let result = invoke_error(&mut host, unapproved);
    assert_eq!(result.code.as_deref(), Some("host_denied"));
    assert!(
        result
            .message
            .as_deref()
            .unwrap_or("")
            .contains("TerminalConsentRequired"),
        "a page boolean must not replace the exact host approval: {result:?}"
    );

    host.backend_mut()
        .host_approve_terminal_command(Path::new(&cwd), "echo hello")
        .expect("host exact-command approval");
    let approved = task_intent(
        "terminal.run",
        json!({"cwd": cwd, "command": "echo hello"}),
        None,
    );
    let result = invoke(&mut host, approved);
    assert_eq!(
        result.status, "ok",
        "unexpected terminal result: {result:?}"
    );
    assert!(
        result.output.as_ref().unwrap()["output"]
            .as_str()
            .unwrap()
            .contains("hello"),
        "the exact approved command must execute: {result:?}"
    );

    let replay = task_intent(
        "terminal.run",
        json!({"cwd": cwd, "command": "echo hello"}),
        None,
    );
    let replay = invoke_error(&mut host, replay);
    assert_eq!(replay.code.as_deref(), Some("host_denied"));

    host.backend_mut()
        .host_approve_terminal_command(Path::new(&cwd), "echo hello")
        .expect("host approval for a different command");
    let different = task_intent(
        "terminal.run",
        json!({"cwd": cwd, "command": "echo goodbye"}),
        None,
    );
    let different = invoke_error(&mut host, different);
    assert_eq!(different.code.as_deref(), Some("host_denied"));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn agent_host_mcp_requires_existing_confirmation() {
    let root = unique_temp_dir("mcp");
    let (mut host, _grant_id) = host_with_grant(&root);
    let mcp = FakeMcp::with_confirmed("native-1");

    let unconfirmed = task_intent(
        "mcp.call",
        json!({"session_id": "native-9", "tool": "echo", "arguments": {}}),
        None,
    );
    let result = invoke_error(&mut host, unconfirmed);
    assert_eq!(result.code.as_deref(), Some("host_denied"));

    host.register_confirmed_mcp_session("native-1");
    let confirmed = task_intent(
        "mcp.call",
        json!({"session_id": "native-1", "tool": "echo", "arguments": {"text": "hi"}}),
        None,
    );
    let result = host
        .invoke(&mcp, confirmed)
        .expect("confirmed MCP session executes");
    assert!(result.status == "ok", "unexpected MCP result: {result:?}");
    assert_eq!(result.output.as_ref().unwrap()["content"][0]["text"], "ok");

    host.unregister_confirmed_mcp_session("native-1");
    let disconnected = task_intent(
        "mcp.call",
        json!({"session_id": "native-1", "tool": "echo", "arguments": {}}),
        None,
    );
    let result = invoke_error(&mut host, disconnected);
    assert_eq!(result.code.as_deref(), Some("host_denied"));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn agent_host_rejects_non_canonical_arguments() {
    let root = unique_temp_dir("non-canonical-args");
    let (mut host, grant_id) = host_with_grant(&root);
    let zero_digest = format!("sha256:{}", "0".repeat(64));

    for arguments in [
        json!({"count": 1.5}),
        json!({"count": -1}),
        json!({"Path": "src/a.ts"}),
        json!({"caf\u{e9}": "value"}),
        json!({"outer": [{"n": 1.25}]}),
    ] {
        let intent = task_intent_raw(
            "file.write",
            arguments.clone(),
            zero_digest.clone(),
            None,
            TASK_A,
            RUN_A,
            3,
            "tc_non_canonical",
        );
        let error = host
            .invoke(&NoMcp, intent)
            .expect_err("non-canonical arguments must be refused before dispatch");
        assert!(
            error.contains("invalid_arguments"),
            "non-canonical arguments must be refused before dispatch: {arguments}: {error}"
        );
    }

    let valid = json!({"grant_id": grant_id, "path": "note.txt", "content": "ok"});
    let wrong_digest = task_intent_raw(
        "file.write",
        valid.clone(),
        zero_digest,
        None,
        TASK_A,
        RUN_A,
        3,
        "tc_wrong_digest",
    );
    let error = host
        .invoke(&NoMcp, wrong_digest)
        .expect_err("a mismatched arguments digest must be refused");
    assert!(error.contains("invalid_arguments"), "unexpected: {error}");
    assert!(!root.join("note.txt").exists());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn agent_host_rejects_non_canonical_results() {
    let root = unique_temp_dir("non-canonical-results");
    let (mut host, _grant_id) = host_with_grant(&root);
    host.register_confirmed_mcp_session("native-1");
    let mcp = FakeMcp {
        confirmed: ["native-1".to_string()].into_iter().collect(),
        non_canonical: true,
    };
    let intent = task_intent(
        "mcp.call",
        json!({"session_id": "native-1", "tool": "echo", "arguments": {}}),
        None,
    );
    let result = host
        .invoke(&mcp, intent)
        .expect("a non-canonical result must be an explicit host error");
    assert_eq!(result.status, "error");
    assert_eq!(result.code.as_deref(), Some("invalid_event"));
    assert!(
        result.output.is_none(),
        "a non-canonical result must not be forwarded to the page: {result:?}"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn agent_host_refuses_out_of_grant_path() {
    let root = unique_temp_dir("out-of-grant");
    let outside = unique_temp_dir("out-of-grant-outside");
    fs::write(outside.join("secret.txt"), "secret").expect("write outside file");
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(&outside, root.join("escape")).expect("symlink escape");
    }
    let (mut host, grant_id) = host_with_grant(&root);
    let absolute_outside = outside.join("secret.txt").display().to_string();

    for path in ["../outside.txt", absolute_outside.as_str()] {
        let intent = task_intent(
            "file.write",
            json!({"grant_id": grant_id, "path": path, "content": "write"}),
            None,
        );
        let result = invoke_error(&mut host, intent);
        assert_eq!(
            result.code.as_deref(),
            Some("host_denied"),
            "out-of-grant path must be refused before approval or write: {path}"
        );
    }

    #[cfg(unix)]
    {
        let symlinked = task_intent(
            "file.write",
            json!({"grant_id": grant_id, "path": "escape/secret.txt", "content": "write"}),
            None,
        );
        let result = invoke_error(&mut host, symlinked);
        assert_eq!(result.code.as_deref(), Some("host_denied"));
        assert_eq!(
            fs::read_to_string(outside.join("secret.txt")).unwrap(),
            "secret"
        );

        let read = task_intent(
            "file.read",
            json!({"grant_id": grant_id, "path": "escape/secret.txt"}),
            None,
        );
        let result = invoke_error(&mut host, read);
        assert_eq!(result.code.as_deref(), Some("host_denied"));
    }

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&outside);
}

#[test]
fn agent_host_refuses_expired_approval() {
    let root = unique_temp_dir("expired");
    let (mut host, grant_id) = host_with_grant(&root);

    host.set_approval_ttl_ms(1);
    let request = write_intent(&grant_id, "note.txt", "value");
    let approval_id = request_approval(&mut host, request);
    std::thread::sleep(Duration::from_millis(10));
    let expired = host.resolve_approval(
        &json!({"approval_id": approval_id, "decision": "approved"}),
        true,
    );
    assert!(
        expired.as_ref().unwrap_err().contains("approval_expired"),
        "unexpected expiry error: {expired:?}"
    );

    host.set_approval_ttl_ms(300);
    let request = write_intent(&grant_id, "note.txt", "value");
    let approval_id = request_approval(&mut host, request);
    approve(&mut host, &approval_id);
    std::thread::sleep(Duration::from_millis(400));
    let execution = task_intent(
        "file.write",
        json!({"grant_id": grant_id, "path": "note.txt", "content": "value"}),
        Some(&approval_id),
    );
    let error = host
        .invoke(&NoMcp, execution)
        .expect_err("an approval that expires before execution must be refused");
    assert!(error.contains("approval_expired"), "unexpected: {error}");
    assert!(!root.join("note.txt").exists());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn agent_host_refuses_digest_mismatched_approval() {
    let root = unique_temp_dir("digest-mismatch");
    let (mut host, grant_id) = host_with_grant(&root);
    let request = write_intent(&grant_id, "note.txt", "first");
    let tool_call_id = request["tool_call_id"]
        .as_str()
        .expect("request tool call")
        .to_string();
    let approval_id = request_approval(&mut host, request);
    approve(&mut host, &approval_id);

    let changed_arguments = json!({"grant_id": grant_id, "path": "note.txt", "content": "second"});
    let changed = task_intent_raw(
        "file.write",
        changed_arguments.clone(),
        digest_of(&changed_arguments),
        Some(&approval_id),
        TASK_A,
        RUN_A,
        3,
        &tool_call_id,
    );
    let error = host
        .invoke(&NoMcp, changed)
        .expect_err("a digest-mismatched approval must be refused");
    assert!(
        error.contains("approval_digest_mismatch"),
        "unexpected: {error}"
    );
    assert!(!root.join("note.txt").exists());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn page_cannot_approve_its_own_valid_intent() {
    let root = unique_temp_dir("self-approve");
    let (mut host, grant_id) = host_with_grant(&root);
    let request = write_intent(&grant_id, "note.txt", "value");
    let approval_id = request_approval(&mut host, request);

    let page_chosen = host.resolve_approval(
        &json!({"approval_id": "apr_aaaaaaaaaaaaaaaa", "decision": "approved"}),
        true,
    );
    assert!(
        page_chosen.unwrap_err().contains("approval_forbidden"),
        "a page-chosen approval identity must not be accepted"
    );

    let page_only = host.resolve_approval(
        &json!({"approval_id": approval_id, "decision": "approved"}),
        false,
    );
    assert!(
        page_only.unwrap_err().contains("approval_forbidden"),
        "the page decision alone must not approve a valid intent"
    );

    let execution = task_intent(
        "file.write",
        json!({"grant_id": grant_id, "path": "note.txt", "content": "value"}),
        Some(&approval_id),
    );
    let pending = host
        .invoke(&NoMcp, execution)
        .expect_err("a pending approval must not execute");
    assert!(pending.contains("approval_forbidden"));
    assert!(!root.join("note.txt").exists());

    let forged_fields = host.resolve_approval(
        &json!({
            "approval_id": approval_id,
            "decision": "approved",
            "granted": true,
        }),
        true,
    );
    assert!(forged_fields.unwrap_err().contains("approval_forbidden"));
    let still_pending = task_intent(
        "file.write",
        json!({"grant_id": grant_id, "path": "note.txt", "content": "value"}),
        Some(&approval_id),
    );
    assert!(host
        .invoke(&NoMcp, still_pending)
        .expect_err("extra authority fields must not approve")
        .contains("approval_forbidden"));

    approve(&mut host, &approval_id);
    let execution = task_intent(
        "file.write",
        json!({"grant_id": grant_id, "path": "note.txt", "content": "value"}),
        Some(&approval_id),
    );
    let result = invoke(&mut host, execution);
    assert_eq!(result.status, "ok");
    assert_eq!(fs::read_to_string(root.join("note.txt")).unwrap(), "value");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn receipt_is_opaque_or_asymmetrically_verified() {
    assert_eq!(
        hmac_sha256_hex(b"key", b"The quick brown fox jumps over the lazy dog"),
        "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8",
        "the host receipt primitive must be HMAC-SHA-256"
    );

    let root = unique_temp_dir("receipt");
    fs::write(root.join("a.txt"), "alpha").expect("write a");
    fs::write(root.join("b.txt"), "bravo").expect("write b");
    let (mut host, grant_id) = host_with_grant(&root);

    let read_intent = |path: &str| {
        task_intent(
            "file.read",
            json!({"grant_id": grant_id, "path": path}),
            None,
        )
    };

    let request_a = read_intent("a.txt");
    let approval_a = request_approval(&mut host, request_a);
    approve(&mut host, &approval_a);
    let intent_a = task_intent(
        "file.read",
        json!({"grant_id": grant_id, "path": "a.txt"}),
        Some(&approval_a),
    );
    let result_a = invoke(&mut host, intent_a.clone());
    let receipt_a = result_a.approval_receipt.clone().expect("receipt a");

    let request_b = read_intent("b.txt");
    let approval_b = request_approval(&mut host, request_b);
    approve(&mut host, &approval_b);
    let intent_b = task_intent(
        "file.read",
        json!({"grant_id": grant_id, "path": "b.txt"}),
        Some(&approval_b),
    );
    let result_b = invoke(&mut host, intent_b.clone());
    let receipt_b = result_b.approval_receipt.clone().expect("receipt b");

    for receipt in [&receipt_a, &receipt_b] {
        assert!(receipt.starts_with("hmac-sha256:"));
        assert_eq!(receipt.len(), "hmac-sha256:".len() + 64);
        assert!(receipt["hmac-sha256:".len()..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
    }
    assert_ne!(receipt_a, receipt_b);
    assert!(host.receipt_is_valid_for(&intent_a, &receipt_a));
    assert!(!host.receipt_is_valid_for(&intent_b, &receipt_a));
    assert!(!host.receipt_is_valid_for(&intent_a, &receipt_b));

    let public_identity = json!({
        "intent_id": intent_a["intent_id"],
        "task_id": intent_a["task_id"],
        "run_id": intent_a["run_id"],
        "generation": intent_a["generation"],
        "tool_call_id": intent_a["tool_call_id"],
        "arguments_digest": intent_a["arguments_digest"],
        "approval_id": intent_a["approval_id"],
    });
    let public_digest = format!(
        "sha256:{}",
        uor_r4_desktop_client::sha256_bytes(
            uor_r4_studio_agent::canonical_json(&public_identity)
                .unwrap()
                .as_bytes()
        )
    );
    assert_ne!(
        public_digest["sha256:".len()..],
        receipt_a["hmac-sha256:".len()..],
        "the receipt must not be a digest of public intent fields"
    );

    let serialized = serde_json::to_value(&result_a).expect("serialize receipt result");
    let object = serialized.as_object().expect("result object");
    for forbidden in ["key", "secret", "hmac_key", "approval_key", "signing_key"] {
        assert!(
            !object.contains_key(forbidden),
            "a host receipt must never serialize {forbidden}: {serialized}"
        );
    }
    let debug = format!("{host:?}");
    assert!(
        debug.contains("redacted"),
        "host debug must redact the receipt key: {debug}"
    );
    assert!(
        !debug.contains(&receipt_a),
        "host debug must not expose the receipt value: {debug}"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn agent_host_file_tools_register_host_artifacts() {
    let root = unique_temp_dir("artifact");
    fs::write(root.join("notes.txt"), "hello artifact").expect("write notes");
    let (mut host, grant_id) = host_with_grant(&root);

    let request = task_intent(
        "file.read",
        json!({"grant_id": grant_id, "path": "notes.txt"}),
        None,
    );
    let approval_id = request_approval(&mut host, request);
    approve(&mut host, &approval_id);
    let read = task_intent(
        "file.read",
        json!({"grant_id": grant_id, "path": "notes.txt"}),
        Some(&approval_id),
    );
    let result = invoke(&mut host, read);
    assert_eq!(result.status, "ok", "unexpected read result: {result:?}");
    let output = result.output.as_ref().expect("read output");
    assert_eq!(output["path"], "notes.txt");
    assert_eq!(output["media_type"], "text/plain");
    assert_eq!(output["size_bytes"], 14);
    let artifact = result.artifact.expect("file read registers an artifact");
    assert!(artifact.artifact_id.starts_with("art_"));
    assert_eq!(artifact.artifact_id.len(), "art_".len() + 64);
    assert!(artifact.artifact_id["art_".len()..]
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
    assert_eq!(artifact.digest, output["digest"]);
    assert_eq!(artifact.media_type, "text/plain");
    assert_eq!(artifact.size_bytes, 14);

    let ranged = data_intent(
        "artifact.read",
        json!({"artifact_id": artifact.artifact_id, "start": 6, "end": 14}),
    );
    let ranged = invoke(&mut host, ranged);
    assert_eq!(ranged.status, "ok");
    let output = ranged.output.expect("artifact bytes");
    assert_eq!(output["bytes_base64"], "YXJ0aWZhY3Q=");
    assert_eq!(output["size_bytes"], 8);
    assert_eq!(output["artifact_size_bytes"], 14);
    assert_eq!(output["range"]["start"], 6);
    assert_eq!(output["range"]["end"], 14);
    assert_eq!(output["whole_digest"], artifact.digest);

    let full = data_intent(
        "artifact.read",
        json!({"artifact_id": artifact.artifact_id}),
    );
    let full = invoke(&mut host, full);
    assert_eq!(full.output.unwrap()["bytes_base64"], "aGVsbG8gYXJ0aWZhY3Q=");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn agent_host_git_changes_passes_snapshot_wrapper() {
    let repo = unique_temp_dir("git-changes");
    let git = |args: &[&str]| {
        let output = Command::new("git")
            .args(args)
            .current_dir(&repo)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_COUNT", "0")
            .output()
            .expect("run git fixture");
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    };
    git(&["init", "-q"]);
    git(&["symbolic-ref", "HEAD", "refs/heads/main"]);
    git(&["config", "user.email", "studio@example.invalid"]);
    git(&["config", "user.name", "Studio Fixture"]);
    git(&["config", "commit.gpgsign", "false"]);
    fs::write(repo.join("base.txt"), "base\n").expect("write base");
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "base"]);
    git(&["checkout", "-q", "-b", "feature"]);
    fs::write(repo.join("feature.txt"), "feature\n").expect("write feature");
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "feature"]);

    let (mut host, grant_id) = host_with_grant(&repo);
    let snapshot = data_intent(
        "git.changes",
        json!({"grant_id": grant_id, "scope": "branch"}),
    );
    let result = invoke(&mut host, snapshot);
    assert_eq!(result.status, "ok", "unexpected diff result: {result:?}");
    let output = result.output.as_ref().expect("diff output");
    let object = output.as_object().expect("diff output object");
    assert!(object.contains_key("snapshot"));
    assert!(!object.contains_key("files"));
    assert!(!object.contains_key("scope"));
    let snapshot = &output["snapshot"];
    assert_eq!(snapshot["scope"], "branch");
    assert_eq!(snapshot["files"][0]["path"], "feature.txt");
    assert_eq!(snapshot["head_digest"].as_str().unwrap().len(), 71);

    let last_turn = data_intent(
        "git.changes",
        json!({"grant_id": grant_id, "scope": "last_turn"}),
    );
    let result = invoke_error(&mut host, last_turn);
    assert_eq!(result.code.as_deref(), Some("invalid_arguments"));

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn agent_host_poll_replays_after_cursor() {
    let root = unique_temp_dir("poll");
    fs::write(root.join("a.txt"), "a").expect("write a");
    let (mut host, grant_id) = host_with_grant(&root);
    let first = data_intent("file.tree", json!({"grant_id": grant_id}));
    let second = data_intent("file.tree", json!({"grant_id": grant_id}));
    let first_id = first["intent_id"].clone();
    let second_id = second["intent_id"].clone();
    invoke(&mut host, first);
    invoke(&mut host, second);

    let all = host.poll(None).expect("poll all results");
    assert_eq!(all["results"].as_array().unwrap().len(), 2);
    assert_eq!(all["cursor"], json!("2"));

    let after_first = host.poll(Some("1")).expect("poll after cursor");
    let results = after_first["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["intent_id"], second_id);
    assert_ne!(results[0]["intent_id"], first_id);
    assert_eq!(after_first["cursor"], json!("2"));

    let after_last = host.poll(Some("2")).expect("poll after last result");
    assert_eq!(after_last["results"], json!([]));

    let malformed = host.poll(Some("not-a-cursor"));
    assert!(
        malformed.unwrap_err().contains("invalid_arguments"),
        "a malformed cursor must be refused"
    );

    let rpc_poll = handle_rpc_line_with_host(
        &mut host,
        &json!({
            "id": 9,
            "command": "agent_host_poll",
            "args": {"cursor": "1"},
        })
        .to_string(),
    );
    assert_eq!(rpc_poll["ok"], json!(true));
    assert_eq!(rpc_poll["result"]["results"].as_array().unwrap().len(), 1);
    let rpc_subscribe = handle_rpc_line_with_host(
        &mut host,
        &json!({"id": 10, "command": "agent_subscribe", "args": {}}).to_string(),
    );
    assert_eq!(rpc_subscribe["ok"], json!(false));
    assert!(
        rpc_subscribe["error"]
            .as_str()
            .unwrap()
            .contains("agent_host_poll"),
        "the offline shim must not assume a Channel: {rpc_subscribe}"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn agent_host_capabilities_expose_grants() {
    let root = unique_temp_dir("capabilities");
    let (host, grant_id) = host_with_grant(&root);
    let capabilities = host.capabilities();
    assert_eq!(capabilities["tree"], json!(true));
    assert_eq!(capabilities["changes"], json!(true));
    assert_eq!(capabilities["readArtifact"], json!(true));
    assert_eq!(capabilities["terminal"], json!(false));
    assert_eq!(capabilities["git"], json!(true));
    assert_eq!(capabilities["mcp"], json!(true));
    assert_eq!(capabilities["approvals"], json!(true));
    assert_eq!(capabilities["runtime"], json!(false));
    let grants = capabilities["grants"].as_array().expect("grant list");
    assert_eq!(grants.len(), 1);
    assert_eq!(grants[0]["grant_id"], grant_id);
    assert_eq!(
        grants[0]["path"],
        fs::canonicalize(&root)
            .unwrap()
            .to_string_lossy()
            .to_string()
    );

    let mut host = host;
    host.backend_mut().native_set_terminal_enabled(true);
    assert_eq!(host.capabilities()["terminal"], json!(true));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn agent_host_has_no_task_runtime() {
    let root = unique_temp_dir("no-runtime");
    let (mut host, _grant_id) = host_with_grant(&root);

    let capabilities = host.capabilities();
    let object = capabilities.as_object().expect("capabilities object");
    assert_eq!(object.get("runtime"), Some(&json!(false)));
    for forbidden in ["tasks", "events", "event_log", "task_store", "run_state"] {
        assert!(
            !object.contains_key(forbidden),
            "the host must not expose task runtime state: {forbidden}"
        );
    }

    for tool in ["task.create", "run.start", "event.append", "task.list"] {
        let intent = task_intent(tool, json!({}), None);
        let result = invoke_error(&mut host, intent);
        assert_eq!(
            result.code.as_deref(),
            Some("invalid_arguments"),
            "the host must not execute task runtime tools: {tool}"
        );
    }

    let data = data_intent("file.tree", json!({}));
    let result = invoke(&mut host, data);
    let serialized = serde_json::to_value(&result).expect("serialize host result");
    assert_eq!(serialized["schema"], json!(HOST_RESULT_SCHEMA));
    for forbidden in ["type", "event_id", "sequence", "recorded_at_ms"] {
        assert!(
            serialized.get(forbidden).is_none(),
            "HostToolResult must not be an AgentEvent: {forbidden}"
        );
    }

    let polled = host.poll(None).expect("host result poll");
    assert!(polled.get("tasks").is_none());
    assert!(polled.get("events").is_none());

    let unknown = handle_rpc_line_with_host(
        &mut host,
        &json!({"id": 1, "command": "agent_host_status", "args": {}}).to_string(),
    );
    assert_eq!(unknown["ok"], json!(false));

    let backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    let fresh = AgentHost::new(backend).expect("secure host");
    let empty = fresh.poll(None).expect("fresh host poll");
    assert_eq!(
        empty["results"],
        json!([]),
        "host results must not persist as a task log"
    );

    let _ = fs::remove_dir_all(&root);
}

#[cfg(feature = "gui")]
#[test]
fn tauri_subscribe_channel_emits_host_results() {
    use tauri::ipc::{Channel, InvokeResponseBody};
    use uor_r4_desktop_client::tauri_gui::{
        agent_host_invoke_on, agent_host_subscribe_on, TauriDesktopState,
    };

    let root = unique_temp_dir("channel");
    fs::write(root.join("a.txt"), "a").expect("write a");
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend.native_grant_folder(&root).expect("grant");
    let grant_id = backend.agent_grant_id(&root).expect("grant id");
    let state = TauriDesktopState::new(backend).expect("host");

    let (sender, receiver) = std::sync::mpsc::channel();
    let channel = Channel::new(move |body: InvokeResponseBody| {
        if let InvokeResponseBody::Json(text) = body {
            let _ = sender.send(text);
        }
        Ok(())
    });
    agent_host_subscribe_on(&state, channel).expect("subscribe channel");

    let intent = data_intent("file.tree", json!({"grant_id": grant_id}));
    let intent_id = intent["intent_id"].clone();
    let direct = agent_host_invoke_on(&state, intent).expect("invoke through host state");
    assert_eq!(direct["status"], json!("ok"));

    let text = receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("channel must receive the host result");
    let result: HostToolResult = serde_json::from_str(&text).expect("channel result JSON");
    assert_eq!(result.schema, HOST_RESULT_SCHEMA);
    assert_eq!(result.intent_id, intent_id);
    assert_eq!(result.status, "ok");
    for forbidden in ["type", "event_id", "sequence", "recorded_at_ms"] {
        assert!(
            serde_json::to_value(&result)
                .unwrap()
                .get(forbidden)
                .is_none(),
            "the channel must carry HostToolResult, not an AgentEvent"
        );
    }

    let second = data_intent("file.tree", json!({"grant_id": grant_id}));
    let second_id = second["intent_id"].clone();
    let second_result = agent_host_invoke_on(&state, second).expect("second invoke");
    assert_eq!(second_result["intent_id"], second_id);
    let second_text = receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("channel must remain open for later results");
    let second: HostToolResult = serde_json::from_str(&second_text).expect("second result");
    assert_eq!(second.intent_id, second_id);

    let _ = fs::remove_dir_all(&root);
}
