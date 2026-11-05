use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use reqwest::blocking::{Client, Response};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use reqwest::redirect::Policy;
use serde_json::{json, Value};

const PROTOCOL_VERSION: &str = "2025-03-26";
const MAX_HEADER_BYTES: usize = 64 * 1024;
const MAX_ARGUMENT_BYTES: usize = 64 * 1024;
const MAX_TOOLS: usize = 256;
const MAX_STDIO_ARGS: usize = 32;
const MAX_STDIO_ARG_BYTES: usize = 4096;

#[derive(Clone, Debug)]
pub struct McpLimits {
    pub max_response_bytes: usize,
    pub max_pages: usize,
    pub timeout: Duration,
}

impl Default for McpLimits {
    fn default() -> Self {
        Self {
            max_response_bytes: 1024 * 1024,
            max_pages: 4,
            timeout: Duration::from_secs(10),
        }
    }
}

#[derive(Clone)]
pub struct McpRemoteSession {
    url: String,
    credential: String,
    session_id: String,
    client: Client,
    limits: McpLimits,
    next_id: Arc<AtomicU64>,
}

impl std::fmt::Debug for McpRemoteSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("McpRemoteSession")
            .field("url", &self.url)
            .field("session_id", &self.session_id)
            .finish_non_exhaustive()
    }
}

struct RemoteAnswer {
    result: Option<Value>,
    session_id: String,
}

impl McpRemoteSession {
    pub fn connect(url: &str, credential: &str, limits: McpLimits) -> Result<Self, String> {
        let parsed = reqwest::Url::parse(url).map_err(|error| error.to_string())?;
        if !matches!(parsed.scheme(), "http" | "https") {
            return Err("MCP requires an HTTP or HTTPS URL.".to_string());
        }
        if !parsed.username().is_empty() || parsed.password().is_some() {
            return Err("MCP credentials must not be embedded in the URL.".to_string());
        }
        let client = Client::builder()
            .redirect(Policy::none())
            .timeout(limits.timeout)
            .user_agent("uor-harness/0.1")
            .build()
            .map_err(|error| error.to_string())?;
        let mut session = Self {
            url: parsed.to_string(),
            credential: credential.to_string(),
            session_id: String::new(),
            client,
            limits,
            next_id: Arc::new(AtomicU64::new(0)),
        };
        let initialized = session.request(
            "initialize",
            json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": {"name": "uor-harness", "version": "0.1.0"}
            }),
            false,
        )?;
        if initialized.result.is_none() {
            return Err("MCP initialize returned no result.".to_string());
        }
        session.session_id = initialized.session_id;
        session.request("notifications/initialized", json!({}), true)?;
        Ok(session)
    }

    fn request(
        &self,
        method: &str,
        params: Value,
        notification: bool,
    ) -> Result<RemoteAnswer, String> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        let mut message = json!({"jsonrpc": "2.0", "method": method, "params": params});
        if !notification {
            message["id"] = json!(id);
        }
        let mut request = self
            .client
            .post(&self.url)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json, text/event-stream")
            .header("MCP-Protocol-Version", PROTOCOL_VERSION)
            .header("Mcp-Session-Id", self.session_id.clone())
            .body(message.to_string());
        if !self.credential.is_empty() {
            request = request.header(AUTHORIZATION, format!("Bearer {}", self.credential));
        }
        let response = request.send().map_err(|error| {
            if error.is_timeout() {
                "MCP request timed out.".to_string()
            } else {
                format!("MCP transport failed: {error}")
            }
        })?;
        read_remote_response(response, self.limits.max_response_bytes)
    }

    pub fn list_tools(&self) -> Result<Vec<Value>, String> {
        let mut tools = Vec::new();
        let mut cursor = None;
        for page in 0..self.limits.max_pages {
            let params = cursor
                .as_ref()
                .map_or_else(|| json!({}), |value| json!({"cursor": value}));
            let result = self
                .request("tools/list", params, false)?
                .result
                .ok_or_else(|| "MCP tools/list returned no result.".to_string())?;
            if let Some(list) = result["tools"].as_array() {
                tools.extend(list.iter().cloned());
            }
            if tools.len() > MAX_TOOLS {
                return Err(format!("MCP tool list exceeds the {MAX_TOOLS}-tool cap."));
            }
            let Some(next) = result["nextCursor"]
                .as_str()
                .filter(|value| !value.is_empty())
            else {
                return Ok(tools);
            };
            if page + 1 == self.limits.max_pages {
                return Err(format!(
                    "MCP pagination cap reached ({} pages).",
                    self.limits.max_pages
                ));
            }
            cursor = Some(next.to_string());
        }
        Err("MCP pagination failed.".to_string())
    }

    pub fn call_tool(&self, name: &str, args: &Value) -> Result<Value, String> {
        if serde_json::to_vec(args)
            .map_err(|error| error.to_string())?
            .len()
            > MAX_ARGUMENT_BYTES
        {
            return Err("MCP tool arguments exceed 64 KiB.".to_string());
        }
        self.request(
            "tools/call",
            json!({"name": name, "arguments": args}),
            false,
        )?
        .result
        .ok_or_else(|| "MCP tools/call returned no result.".to_string())
    }
}

fn read_remote_response(response: Response, max_bytes: usize) -> Result<RemoteAnswer, String> {
    let status = response.status();
    let header_bytes = response
        .headers()
        .iter()
        .map(|(name, value)| name.as_str().len() + value.as_bytes().len() + 4)
        .sum::<usize>();
    if header_bytes > MAX_HEADER_BYTES {
        return Err("MCP response headers exceed 64 KiB.".to_string());
    }
    let session_id = response
        .headers()
        .get("mcp-session-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();
    let mut body = Vec::new();
    response
        .take((max_bytes + 1) as u64)
        .read_to_end(&mut body)
        .map_err(|error| format!("MCP response read failed: {error}"))?;
    if body.len() > max_bytes {
        return Err(format!(
            "MCP response exceeds the {max_bytes}-byte size cap."
        ));
    }
    if !status.is_success() {
        let detail = String::from_utf8_lossy(&body);
        return Err(format!(
            "MCP request failed with HTTP {status}: {}",
            &detail.chars().take(400).collect::<String>()
        ));
    }
    if body.is_empty() {
        return Ok(RemoteAnswer {
            result: None,
            session_id,
        });
    }
    let text = String::from_utf8_lossy(&body);
    let payload = parse_payload(&text, &content_type)?;
    if let Some(error) = payload.get("error") {
        return Err(format!(
            "MCP server error: {}",
            error["message"].as_str().unwrap_or("unknown")
        ));
    }
    Ok(RemoteAnswer {
        result: payload.get("result").cloned().or(Some(payload)),
        session_id,
    })
}

fn parse_payload(text: &str, content_type: &str) -> Result<Value, String> {
    if content_type.contains("text/event-stream") {
        let mut latest = None;
        for block in text.split("\r\n\r\n").flat_map(|part| part.split("\n\n")) {
            let data = block
                .lines()
                .filter(|line| line.starts_with("data:"))
                .map(|line| line.trim_start_matches("data:").trim())
                .collect::<Vec<_>>()
                .join("\n");
            if data.is_empty() || data == "[DONE]" {
                continue;
            }
            latest = Some(serde_json::from_str::<Value>(&data).map_err(|error| error.to_string())?);
        }
        return latest.ok_or_else(|| "MCP SSE response contained no JSON result.".to_string());
    }
    if !content_type.contains("application/json") && !content_type.contains("+json") {
        return Err("MCP server returned an unsupported content type.".to_string());
    }
    serde_json::from_str(text).map_err(|error| error.to_string())
}

struct StdioInner {
    child: Mutex<Child>,
    stdin: Mutex<ChildStdin>,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,
    io: Mutex<()>,
}

impl StdioInner {
    fn kill(&self) -> bool {
        self.child
            .lock()
            .map(|mut child| super::process_control::terminate(&mut child))
            .unwrap_or(false)
    }
}

impl Drop for StdioInner {
    fn drop(&mut self) {
        self.kill();
    }
}

pub struct McpStdioSession {
    inner: Arc<StdioInner>,
    limits: McpLimits,
    next_id: AtomicU64,
}

impl std::fmt::Debug for McpStdioSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("McpStdioSession")
            .finish_non_exhaustive()
    }
}

impl McpStdioSession {
    pub fn spawn(
        executable: &Path,
        args: &[String],
        cwd: &Path,
        limits: McpLimits,
    ) -> Result<Self, String> {
        if !executable.is_absolute() || !cwd.is_absolute() {
            return Err(
                "MCP stdio requires exact absolute executable and working-directory paths."
                    .to_string(),
            );
        }
        if args.len() > MAX_STDIO_ARGS || args.iter().any(|arg| arg.len() > MAX_STDIO_ARG_BYTES) {
            return Err("MCP stdio arguments exceed their declared cap.".to_string());
        }
        let canonical_cwd = fs::canonicalize(cwd).map_err(|error| error.to_string())?;
        if !canonical_cwd.is_dir() {
            return Err("MCP stdio working directory is not a directory.".to_string());
        }
        let canonical_executable =
            fs::canonicalize(executable).map_err(|error| error.to_string())?;
        if !canonical_executable.starts_with(&canonical_cwd) {
            return Err("MCP executable is outside the approved working directory.".to_string());
        }
        if fs::symlink_metadata(executable)
            .map_err(|error| error.to_string())?
            .file_type()
            .is_symlink()
        {
            return Err("MCP executable symlinks are refused.".to_string());
        }
        if !canonical_executable.is_file() {
            return Err("MCP executable is not a file.".to_string());
        }
        let mut command = Command::new(&canonical_executable);
        super::process_control::prepare(&mut command);
        let mut child = command
            .args(args)
            .current_dir(&canonical_cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("could not start MCP stdio server: {error}"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "MCP stdin unavailable".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "MCP stdout unavailable".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "MCP stderr unavailable".to_string())?;
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stderr);
            let mut sink = std::io::sink();
            let _ = std::io::copy(&mut reader, &mut sink);
        });
        let session = Self {
            inner: Arc::new(StdioInner {
                child: Mutex::new(child),
                stdin: Mutex::new(stdin),
                stdout: Arc::new(Mutex::new(BufReader::new(stdout))),
                io: Mutex::new(()),
            }),
            limits,
            next_id: AtomicU64::new(0),
        };
        session.request(
            "initialize",
            json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": {"name": "uor-harness", "version": "0.1.0"}
            }),
            false,
        )?;
        session.request("notifications/initialized", json!({}), true)?;
        Ok(session)
    }

    fn request(
        &self,
        method: &str,
        params: Value,
        notification: bool,
    ) -> Result<Option<Value>, String> {
        let _io = self
            .inner
            .io
            .lock()
            .map_err(|_| "MCP stdio I/O mutex poisoned".to_string())?;
        let id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        let mut message = json!({"jsonrpc": "2.0", "method": method, "params": params});
        if !notification {
            message["id"] = json!(id);
        }
        {
            let mut stdin = self
                .inner
                .stdin
                .lock()
                .map_err(|_| "MCP stdin mutex poisoned".to_string())?;
            stdin
                .write_all(message.to_string().as_bytes())
                .and_then(|_| stdin.write_all(b"\n"))
                .and_then(|_| stdin.flush())
                .map_err(|error| format!("MCP stdio write failed: {error}"))?;
        }
        if notification {
            return Ok(None);
        }
        let stdout = self.inner.stdout.clone();
        let (sender, receiver) = std::sync::mpsc::channel();
        let reader = std::thread::spawn(move || {
            let result = read_stdio_result(&stdout, id);
            let _ = sender.send(result);
        });
        match receiver.recv_timeout(self.limits.timeout) {
            Ok(result) => {
                let _ = reader.join();
                result
            }
            Err(_) => {
                if self.inner.kill() {
                    let _ = reader.join();
                }
                Err("MCP stdio request timed out.".to_string())
            }
        }
    }

    pub fn list_tools(&self) -> Result<Vec<Value>, String> {
        let mut tools = Vec::new();
        let mut cursor = None;
        for page in 0..self.limits.max_pages {
            let params = cursor
                .as_ref()
                .map_or_else(|| json!({}), |value| json!({"cursor": value}));
            let result = self
                .request("tools/list", params, false)?
                .ok_or_else(|| "MCP tools/list returned no result.".to_string())?;
            if let Some(list) = result["tools"].as_array() {
                tools.extend(list.iter().cloned());
            }
            if tools.len() > MAX_TOOLS {
                return Err(format!("MCP tool list exceeds the {MAX_TOOLS}-tool cap."));
            }
            let Some(next) = result["nextCursor"]
                .as_str()
                .filter(|value| !value.is_empty())
            else {
                return Ok(tools);
            };
            if page + 1 == self.limits.max_pages {
                return Err(format!(
                    "MCP pagination cap reached ({} pages).",
                    self.limits.max_pages
                ));
            }
            cursor = Some(next.to_string());
        }
        Err("MCP pagination failed.".to_string())
    }

    pub fn call_tool(&self, name: &str, args: &Value) -> Result<Value, String> {
        if serde_json::to_vec(args)
            .map_err(|error| error.to_string())?
            .len()
            > MAX_ARGUMENT_BYTES
        {
            return Err("MCP tool arguments exceed 64 KiB.".to_string());
        }
        self.request(
            "tools/call",
            json!({"name": name, "arguments": args}),
            false,
        )?
        .ok_or_else(|| "MCP tools/call returned no result.".to_string())
    }

    pub fn is_running(&self) -> bool {
        self.inner
            .child
            .lock()
            .ok()
            .and_then(|mut child| child.try_wait().ok().map(|status| status.is_none()))
            .unwrap_or(false)
    }
}

fn read_frame(reader: &mut impl BufRead, max_bytes: usize) -> Result<Vec<u8>, String> {
    let mut frame = Vec::new();
    loop {
        let available = reader
            .fill_buf()
            .map_err(|error| format!("MCP stdio read failed: {error}"))?;
        if available.is_empty() {
            return Err("MCP stdio process exited.".to_string());
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let take = newline.map_or(available.len(), |position| position + 1);
        if frame.len() + take > max_bytes {
            return Err(format!("MCP stdio frame exceeds the {max_bytes}-byte cap."));
        }
        frame.extend_from_slice(&available[..take]);
        reader.consume(take);
        if newline.is_some() {
            return Ok(frame);
        }
    }
}

fn read_stdio_result(
    stdout: &Arc<Mutex<BufReader<ChildStdout>>>,
    id: u64,
) -> Result<Option<Value>, String> {
    let mut reader = stdout
        .lock()
        .map_err(|_| "MCP stdout mutex poisoned".to_string())?;
    for _ in 0..16 {
        let frame = read_frame(&mut *reader, 1024 * 1024)?;
        let payload: Value = serde_json::from_slice(&frame)
            .map_err(|error| format!("MCP stdio returned invalid JSON: {error}"))?;
        if payload["id"].as_u64() != Some(id) {
            continue;
        }
        if let Some(error) = payload.get("error") {
            return Err(format!(
                "MCP stdio server error: {}",
                error["message"].as_str().unwrap_or("unknown")
            ));
        }
        return Ok(payload.get("result").cloned().or(Some(payload)));
    }
    Err("MCP stdio response contained too many unrelated frames.".to_string())
}

#[derive(Clone)]
enum McpSession {
    Remote(McpRemoteSession),
    Stdio(Arc<McpStdioSession>),
}

#[derive(Default, Clone)]
pub struct McpManager {
    sessions: Arc<Mutex<BTreeMap<String, McpSession>>>,
    slots: Arc<Mutex<usize>>,
    next_id: Arc<AtomicU64>,
}

impl McpManager {
    fn reserve(&self) -> Result<(), String> {
        let mut slots = self
            .slots
            .lock()
            .map_err(|_| "MCP slot mutex poisoned".to_string())?;
        if *slots >= 4 {
            return Err("MCP session cap reached (4).".to_string());
        }
        *slots += 1;
        Ok(())
    }

    fn release_reservation(&self) {
        if let Ok(mut slots) = self.slots.lock() {
            *slots = slots.saturating_sub(1);
        }
    }

    fn insert(&self, session: McpSession) -> Result<String, String> {
        let id = format!(
            "native-{}",
            self.next_id.fetch_add(1, Ordering::Relaxed) + 1
        );
        self.sessions
            .lock()
            .map_err(|_| "MCP session mutex poisoned".to_string())?
            .insert(id.clone(), session);
        Ok(id)
    }

    pub fn connect_remote(
        &self,
        url: &str,
        credential: &str,
        limits: McpLimits,
    ) -> Result<String, String> {
        self.reserve()?;
        match McpRemoteSession::connect(url, credential, limits) {
            Ok(session) => self.insert(McpSession::Remote(session)),
            Err(error) => {
                self.release_reservation();
                Err(error)
            }
        }
    }

    pub fn connect_stdio(
        &self,
        executable: &Path,
        args: &[String],
        cwd: &Path,
        limits: McpLimits,
    ) -> Result<String, String> {
        self.reserve()?;
        match McpStdioSession::spawn(executable, args, cwd, limits) {
            Ok(session) => self.insert(McpSession::Stdio(Arc::new(session))),
            Err(error) => {
                self.release_reservation();
                Err(error)
            }
        }
    }

    fn get(&self, id: &str) -> Result<McpSession, String> {
        self.sessions
            .lock()
            .map_err(|_| "MCP session mutex poisoned".to_string())?
            .get(id)
            .cloned()
            .ok_or_else(|| "MCP session is unavailable.".to_string())
    }

    pub fn disconnect(&self, id: &str) -> Result<(), String> {
        let removed = self
            .sessions
            .lock()
            .map_err(|_| "MCP session mutex poisoned".to_string())?
            .remove(id);
        if removed.is_none() {
            return Err("MCP session is already disconnected.".to_string());
        }
        self.release_reservation();
        Ok(())
    }

    pub fn list_tools(&self, id: &str) -> Result<Vec<Value>, String> {
        let result = match self.get(id)? {
            McpSession::Remote(session) => session.list_tools(),
            McpSession::Stdio(session) => session.list_tools(),
        };
        if result.is_err() {
            let removed = self
                .sessions
                .lock()
                .map_err(|_| "MCP session mutex poisoned".to_string())?
                .remove(id);
            if removed.is_some() {
                self.release_reservation();
            }
        }
        result
    }

    pub fn call_tool(&self, id: &str, name: &str, args: &Value) -> Result<Value, String> {
        match self.get(id)? {
            McpSession::Remote(session) => session.call_tool(name, args),
            McpSession::Stdio(session) => session.call_tool(name, args),
        }
    }
}
