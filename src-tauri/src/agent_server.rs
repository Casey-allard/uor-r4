//! Loopback HTTP intent/result transport on the existing Studio listener.
//!
//! The browser remains the only task/event/cursor owner. This module adds two
//! routes to the already-bound Studio listener:
//!
//! - `POST /v1/studio/host/intents` accepts one canonical host intent and
//!   settles it through the existing [`crate::agent_host::AgentHost`], after
//!   which the browser receives the result as an ordinary `HostToolResult`.
//! - `GET /v1/studio/host/results` streams canonical `HostToolResult` values
//!   as NDJSON from the host's existing bounded result queue.
//!
//! There is no second listener, TLS/remote bind, URL token, EventSource/SSE,
//! host task runtime, event log or result cursor authority here. The stream is
//! ordinary `fetch` + `ReadableStream` data; a held stream never blocks an
//! intent POST because each accepted connection is served independently on the
//! same listener.

use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

use serde_json::{json, Value};

use crate::agent_host::{
    AgentHost, HostToolResult, UnavailableMcpDispatch, MAX_HOST_ARGUMENT_BYTES,
};

pub const HOST_INTENTS_PATH: &str = "/v1/studio/host/intents";
pub const HOST_RESULTS_PATH: &str = "/v1/studio/host/results";
pub const NDJSON_CONTENT_TYPE: &str = "application/x-ndjson; charset=utf-8";

const MAX_REQUEST_HEAD_BYTES: usize = 16 * 1024;
const MAX_INTENT_BODY_BYTES: usize = MAX_HOST_ARGUMENT_BYTES + 256 * 1024;
const MAX_OTHER_BODY_BYTES: usize = 32 * 1024 * 1024;
const ACCEPT_POLL_MS: u64 = 2;
const REQUEST_READ_TIMEOUT_MS: u64 = 10_000;
const STREAM_WRITE_TIMEOUT_MS: u64 = 5_000;
const STREAM_SLICE_MS: u64 = 200;
const PEER_PEEK_MS: u64 = 1;

/// One loopback Studio listener carrying the existing static/`__tauri_invoke`
/// surfaces plus the two host routes.
pub struct StudioServer {
    listener: TcpListener,
    studio_root: PathBuf,
    bound_host: String,
    expected_token: String,
    host: Arc<Mutex<AgentHost>>,
    shutdown: Arc<AtomicBool>,
}

impl StudioServer {
    /// Bind the one loopback listener. A non-loopback address is refused before
    /// any request is served; the browser and host never speak TLS here.
    pub fn start(
        bind_addr: &str,
        studio_root: impl Into<PathBuf>,
        host: Arc<Mutex<AgentHost>>,
        expected_token: String,
    ) -> Result<Self, String> {
        let listener =
            TcpListener::bind(bind_addr).map_err(|error| format!("bind {bind_addr}: {error}"))?;
        let local = listener.local_addr().map_err(|error| error.to_string())?;
        if !local.ip().is_loopback() {
            return Err(format!(
                "refusing non-loopback bind address {local}; use 127.0.0.1:<port>"
            ));
        }
        listener
            .set_nonblocking(true)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            listener,
            studio_root: studio_root.into(),
            bound_host: local.to_string(),
            expected_token,
            host,
            shutdown: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn bound_host(&self) -> &str {
        &self.bound_host
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    pub fn shutdown_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.shutdown)
    }

    /// Serve until the shutdown flag is set. Each accepted connection gets its
    /// own thread on this same listener; a stream that waits for results does
    /// not block intent posts or static assets.
    pub fn serve(self) {
        loop {
            if self.shutdown.load(Ordering::SeqCst) {
                return;
            }
            match self.listener.accept() {
                Ok((stream, _peer)) => {
                    let host = Arc::clone(&self.host);
                    let studio_root = self.studio_root.clone();
                    let bound_host = self.bound_host.clone();
                    let expected_token = self.expected_token.clone();
                    let shutdown = Arc::clone(&self.shutdown);
                    thread::spawn(move || {
                        if let Err(error) = serve_connection(
                            stream,
                            &studio_root,
                            &bound_host,
                            &expected_token,
                            host,
                            shutdown,
                        ) {
                            eprintln!("studio loopback connection error: {error}");
                        }
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(ACCEPT_POLL_MS));
                }
                Err(_) => return,
            }
        }
    }
}

struct RequestHead {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    buffered: Vec<u8>,
    content_length: Option<usize>,
    clean_path: String,
}

struct RequestFailure {
    status: u16,
    code: &'static str,
    message: String,
}

impl RequestFailure {
    fn new(status: u16, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }
}

fn serve_connection(
    mut stream: TcpStream,
    studio_root: &Path,
    bound_host: &str,
    expected_token: &str,
    host: Arc<Mutex<AgentHost>>,
    shutdown: Arc<AtomicBool>,
) -> Result<(), String> {
    let _ = stream.set_nonblocking(false);
    let _ = stream.set_read_timeout(Some(Duration::from_millis(REQUEST_READ_TIMEOUT_MS)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(STREAM_WRITE_TIMEOUT_MS)));

    let head = match read_request_head(&mut stream) {
        Ok(head) => head,
        Err(failure) => {
            let _ = write_json_error(&mut stream, failure.status, failure.code, &failure.message);
            return Ok(());
        }
    };
    let RequestHead {
        method,
        path,
        headers,
        buffered,
        content_length,
        clean_path,
    } = head;

    if clean_path == HOST_RESULTS_PATH {
        if method != "GET" {
            return write_json_error(
                &mut stream,
                405,
                "invalid_arguments",
                "the host result stream is GET-only",
            );
        }
        // The host routes authorize before any declared body is read, so an
        // unauthenticated local process cannot force a large allocation.
        if let Err(failure) = authorize(&headers, bound_host, expected_token) {
            return write_json_error(&mut stream, failure.status, failure.code, &failure.message);
        }
        if has_query_parameter(&path, "token") {
            return write_json_error(
                &mut stream,
                403,
                "host_denied",
                "tokens are accepted only in the X-UOR-Desktop-Token header",
            );
        }
        return serve_results(
            stream,
            host,
            shutdown,
            bound_host,
            expected_token,
            &path,
            &headers,
        );
    }
    if clean_path == HOST_INTENTS_PATH {
        if method != "POST" {
            return write_json_error(
                &mut stream,
                405,
                "invalid_arguments",
                "host intents are POST-only",
            );
        }
        if let Err(failure) = authorize(&headers, bound_host, expected_token) {
            return write_json_error(&mut stream, failure.status, failure.code, &failure.message);
        }
        if has_query_parameter(&path, "token") {
            return write_json_error(
                &mut stream,
                403,
                "host_denied",
                "tokens are accepted only in the X-UOR-Desktop-Token header",
            );
        }
        if !json_content_type(&headers) {
            return write_json_error(
                &mut stream,
                415,
                "invalid_arguments",
                "host intent POST requires application/json",
            );
        }
        let body = match read_declared_body(
            &mut stream,
            buffered,
            content_length,
            MAX_INTENT_BODY_BYTES,
            true,
        ) {
            Ok(body) => body,
            Err(failure) => {
                return write_json_error(
                    &mut stream,
                    failure.status,
                    failure.code,
                    &failure.message,
                )
            }
        };
        return serve_intent(
            stream,
            host,
            bound_host,
            expected_token,
            &path,
            &headers,
            body,
        );
    }

    let body = match read_declared_body(
        &mut stream,
        buffered,
        content_length,
        MAX_OTHER_BODY_BYTES,
        false,
    ) {
        Ok(body) => body,
        Err(failure) => {
            return write_json_error(&mut stream, failure.status, failure.code, &failure.message)
        }
    };
    let (status, content_type, response) = {
        let mut guard = lock_host(&host);
        crate::handle_studio_http_request_with_host(
            &mut guard,
            studio_root,
            bound_host,
            expected_token,
            &method,
            &path,
            &headers,
            &body,
        )
    };
    write_http_response(&mut stream, status, content_type, &[], &response)
        .map_err(|error| error.to_string())
}

fn read_head_bytes(stream: &mut TcpStream) -> Result<(Vec<u8>, Vec<u8>), RequestFailure> {
    let mut buffer = Vec::new();
    loop {
        if let Some(position) = find_subslice(&buffer, b"\r\n\r\n") {
            let rest = buffer.split_off(position + 4);
            return Ok((buffer, rest));
        }
        if buffer.len() > MAX_REQUEST_HEAD_BYTES {
            return Err(RequestFailure::new(
                413,
                "limit_exceeded",
                "request headers exceed the loopback host limit",
            ));
        }
        let mut chunk = [0u8; 4096];
        match stream.read(&mut chunk) {
            Ok(0) => {
                return Err(RequestFailure::new(
                    400,
                    "invalid_arguments",
                    "connection closed before request headers",
                ))
            }
            Ok(read) => buffer.extend_from_slice(&chunk[..read]),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                return Err(RequestFailure::new(
                    400,
                    "invalid_arguments",
                    "request headers timed out",
                ))
            }
            Err(error) if error.kind() == io::ErrorKind::TimedOut => {
                return Err(RequestFailure::new(
                    400,
                    "invalid_arguments",
                    "request headers timed out",
                ))
            }
            Err(_) => {
                return Err(RequestFailure::new(
                    400,
                    "invalid_arguments",
                    "request headers could not be read",
                ))
            }
        }
    }
}

fn read_request_head(stream: &mut TcpStream) -> Result<RequestHead, RequestFailure> {
    let (head, buffered) = read_head_bytes(stream)?;
    let (method, path, headers) = parse_request_head(&head)?;

    // Every Transfer-Encoding header must be identity. Checking only the first
    // would let `identity` followed by `chunked` through.
    let encodings: Vec<&str> = headers
        .iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case("transfer-encoding"))
        .map(|(_, value)| value.trim())
        .collect();
    if encodings
        .iter()
        .any(|encoding| !encoding.eq_ignore_ascii_case("identity"))
    {
        return Err(RequestFailure::new(
            411,
            "invalid_arguments",
            "Transfer-Encoding must be identity on every header",
        ));
    }

    let content_lengths: Vec<&str> = headers
        .iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .map(|(_, value)| value.trim())
        .collect();
    if content_lengths.len() > 1 {
        return Err(RequestFailure::new(
            400,
            "invalid_arguments",
            "duplicate Content-Length headers are refused",
        ));
    }
    let content_length = match content_lengths.first() {
        None => None,
        Some(value) => Some(value.parse::<usize>().map_err(|_| {
            RequestFailure::new(
                400,
                "invalid_arguments",
                "Content-Length must be a non-negative integer",
            )
        })?),
    };
    let clean_path = path.split('?').next().unwrap_or("/").to_string();
    Ok(RequestHead {
        method,
        path,
        headers,
        buffered,
        content_length,
        clean_path,
    })
}

fn read_declared_body(
    stream: &mut TcpStream,
    mut buffered: Vec<u8>,
    content_length: Option<usize>,
    cap: usize,
    require_length: bool,
) -> Result<Vec<u8>, RequestFailure> {
    let Some(length) = content_length else {
        if require_length {
            return Err(RequestFailure::new(
                411,
                "invalid_arguments",
                "host intent POST requires Content-Length",
            ));
        }
        return Ok(Vec::new());
    };
    if length > cap {
        return Err(RequestFailure::new(
            413,
            "limit_exceeded",
            "request body exceeds the loopback host limit",
        ));
    }
    buffered.reserve(length.saturating_sub(buffered.len()));
    while buffered.len() < length {
        let mut chunk = [0u8; 16 * 1024];
        match stream.read(&mut chunk) {
            Ok(0) => {
                return Err(RequestFailure::new(
                    400,
                    "invalid_arguments",
                    "request body ended before Content-Length",
                ))
            }
            Ok(read) => buffered.extend_from_slice(&chunk[..read]),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                return Err(RequestFailure::new(
                    400,
                    "invalid_arguments",
                    "request body read timed out",
                ))
            }
            Err(error) if error.kind() == io::ErrorKind::TimedOut => {
                return Err(RequestFailure::new(
                    400,
                    "invalid_arguments",
                    "request body read timed out",
                ))
            }
            Err(_) => {
                return Err(RequestFailure::new(
                    400,
                    "invalid_arguments",
                    "request body could not be read",
                ))
            }
        }
    }
    buffered.truncate(length);
    Ok(buffered)
}

type ParsedHead = (String, String, Vec<(String, String)>);

fn parse_request_head(head: &[u8]) -> Result<ParsedHead, RequestFailure> {
    let text = String::from_utf8_lossy(head);
    let mut lines = text.split("\r\n");
    let request_line = lines.next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("").to_string();
    let version = parts.next().unwrap_or("");
    if method.is_empty()
        || method.len() > 16
        || !method.bytes().all(|byte| byte.is_ascii_uppercase())
        || !path.starts_with('/')
        || !version.starts_with("HTTP/")
    {
        return Err(RequestFailure::new(
            400,
            "invalid_arguments",
            "malformed HTTP request line",
        ));
    }
    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let Some((name, value)) = line.split_once(':') else {
            return Err(RequestFailure::new(
                400,
                "invalid_arguments",
                "malformed HTTP header",
            ));
        };
        headers.push((name.trim().to_string(), value.trim().to_string()));
    }
    Ok((method, path, headers))
}

fn serve_results(
    mut stream: TcpStream,
    host: Arc<Mutex<AgentHost>>,
    shutdown: Arc<AtomicBool>,
    bound_host: &str,
    expected_token: &str,
    path: &str,
    headers: &[(String, String)],
) -> Result<(), String> {
    if let Err(failure) = authorize(headers, bound_host, expected_token) {
        return write_json_error(&mut stream, failure.status, failure.code, &failure.message);
    }
    if has_query_parameter(path, "token") {
        return write_json_error(
            &mut stream,
            403,
            "host_denied",
            "tokens are accepted only in the X-UOR-Desktop-Token header",
        );
    }
    let cursor = match cursor_parameter(path) {
        Ok(cursor) => cursor,
        Err(message) => return write_json_error(&mut stream, 400, "invalid_arguments", &message),
    };
    let requested_cursor = cursor
        .as_deref()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);

    let (backlog, subscription_id, receiver) = {
        let mut guard = lock_host(&host);
        match guard.poll_results(cursor.as_deref()) {
            Ok((_latest, backlog)) => {
                let (sender, receiver) = mpsc::channel::<HostToolResult>();
                let (subscription_id, _) = guard
                    .subscribe_with_id(Box::new(move |result| {
                        let _ = sender.send(result.clone());
                    }))
                    .map_err(|error| error.to_string())?;
                (backlog, subscription_id, receiver)
            }
            Err(error) => {
                return write_json_error(
                    &mut stream,
                    cursor_error_status(&error),
                    crate::agent_host::error_code(&error),
                    &error,
                );
            }
        }
    };

    // The header advertises only the cursor the client asked for. Advancing it
    // to `latest` here would let a disconnect before backlog replay skip
    // unprocessed results on resume.
    if write_stream_headers(&mut stream, requested_cursor).is_err() {
        lock_host(&host).unsubscribe(subscription_id);
        return Ok(());
    }
    let mut last_written = requested_cursor;
    for result in backlog {
        if write_result_line(&mut stream, &result).is_err() {
            lock_host(&host).unsubscribe(subscription_id);
            return Ok(());
        }
        last_written = last_written.max(result_sequence(&result));
    }
    loop {
        if shutdown.load(Ordering::SeqCst) {
            break;
        }
        match receiver.recv_timeout(Duration::from_millis(STREAM_SLICE_MS)) {
            Ok(result) => {
                if result_sequence(&result) <= last_written {
                    continue;
                }
                if write_result_line(&mut stream, &result).is_err() {
                    break;
                }
                last_written = result_sequence(&result);
            }
            Err(RecvTimeoutError::Timeout) => {
                if peer_closed(&mut stream) {
                    break;
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    lock_host(&host).unsubscribe(subscription_id);
    Ok(())
}

fn serve_intent(
    mut stream: TcpStream,
    host: Arc<Mutex<AgentHost>>,
    bound_host: &str,
    expected_token: &str,
    path: &str,
    headers: &[(String, String)],
    body: Vec<u8>,
) -> Result<(), String> {
    if let Err(failure) = authorize(headers, bound_host, expected_token) {
        return write_json_error(&mut stream, failure.status, failure.code, &failure.message);
    }
    if has_query_parameter(path, "token") {
        return write_json_error(
            &mut stream,
            403,
            "host_denied",
            "tokens are accepted only in the X-UOR-Desktop-Token header",
        );
    }
    let json_content = json_content_type(headers);
    if !json_content {
        return write_json_error(
            &mut stream,
            415,
            "invalid_arguments",
            "host intent POST requires application/json",
        );
    }
    let intent: Value = match serde_json::from_slice(&body) {
        Ok(intent) => intent,
        Err(_) => {
            return write_json_error(
                &mut stream,
                400,
                "invalid_arguments",
                "host intent body is not JSON",
            )
        }
    };
    if let Err(error) = AgentHost::validate_intent(&intent) {
        return write_json_error(
            &mut stream,
            400,
            crate::agent_host::error_code(&error),
            &error,
        );
    }
    let intent_id = intent
        .get("intent_id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    // Acceptance is not execution. The existing AgentHost remains the only
    // executor, approval authority and bounded result queue; this thread only
    // waits for the host mutex instead of holding the browser POST open.
    thread::spawn(move || {
        let mut guard = lock_host(&host);
        let _ = guard.invoke_transport(&UnavailableMcpDispatch, intent);
    });

    let body = serde_json::to_vec(&json!({"accepted": true, "intent_id": intent_id}))
        .map_err(|error| error.to_string())?;
    write_http_response(
        &mut stream,
        202,
        "application/json; charset=utf-8",
        &[("Cache-Control", "no-store")],
        &body,
    )
    .map_err(|error| error.to_string())
}

fn authorize(
    headers: &[(String, String)],
    bound_host: &str,
    expected_token: &str,
) -> Result<(), RequestFailure> {
    let host_headers: Vec<&str> = headers
        .iter()
        .filter(|(key, _)| key.eq_ignore_ascii_case("host"))
        .map(|(_, value)| value.trim())
        .collect();
    let Some(request_host) = host_headers.first() else {
        return Err(RequestFailure::new(
            403,
            "host_denied",
            "Forbidden: missing Host header",
        ));
    };
    if host_headers.len() != 1 || !request_host.eq_ignore_ascii_case(bound_host) {
        return Err(RequestFailure::new(
            403,
            "host_denied",
            "Forbidden: Host header mismatch",
        ));
    }
    let token_headers: Vec<&str> = headers
        .iter()
        .filter(|(key, _)| key.eq_ignore_ascii_case("x-uor-desktop-token"))
        .map(|(_, value)| value.trim())
        .collect();
    let token_ok = !expected_token.is_empty()
        && token_headers.len() == 1
        && constant_time_eq(token_headers[0], expected_token);
    if !token_ok {
        return Err(RequestFailure::new(
            403,
            "host_denied",
            "Forbidden: invalid X-UOR-Desktop-Token",
        ));
    }
    let origins: Vec<&str> = headers
        .iter()
        .filter(|(key, _)| key.eq_ignore_ascii_case("origin"))
        .map(|(_, value)| value.trim())
        .collect();
    if origins.len() > 1 {
        return Err(RequestFailure::new(
            403,
            "host_denied",
            "Forbidden: duplicate Origin headers",
        ));
    }
    if let Some(origin) = origins.first() {
        let expected_origin = format!("http://{bound_host}");
        if !origin.eq_ignore_ascii_case(&expected_origin) {
            return Err(RequestFailure::new(
                403,
                "host_denied",
                "Forbidden: Origin is not the bound loopback origin",
            ));
        }
    }
    Ok(())
}

fn find_header<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.trim())
}

fn json_content_type(headers: &[(String, String)]) -> bool {
    find_header(headers, "content-type").is_some_and(|value| {
        value
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .eq_ignore_ascii_case("application/json")
    })
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

fn has_query_parameter(path: &str, name: &str) -> bool {
    query_pairs(path).any(|(key, _)| key == name)
}

fn query_pairs(path: &str) -> impl Iterator<Item = (&str, &str)> {
    path.split_once('?')
        .map(|(_, query)| query)
        .unwrap_or("")
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| pair.split_once('=').unwrap_or((pair, "")))
}

fn cursor_parameter(path: &str) -> Result<Option<String>, String> {
    for (key, value) in query_pairs(path) {
        if key != "cursor" {
            continue;
        }
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err("cursor must be a non-negative integer".to_string());
        }
        return Ok(Some(value.to_string()));
    }
    Ok(None)
}

fn cursor_error_status(error: &str) -> u16 {
    if error.starts_with("cursor_expired") {
        410
    } else if error.starts_with("out_of_order") {
        409
    } else {
        400
    }
}

fn result_sequence(result: &HostToolResult) -> u64 {
    result
        .cursor
        .as_deref()
        .and_then(|cursor| cursor.parse::<u64>().ok())
        .unwrap_or(0)
}

fn peer_closed(stream: &mut TcpStream) -> bool {
    let _ = stream.set_read_timeout(Some(Duration::from_millis(PEER_PEEK_MS)));
    let mut byte = [0u8; 1];
    match stream.peek(&mut byte) {
        Ok(0) => true,
        Ok(_) => false,
        Err(error) => !matches!(
            error.kind(),
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut | io::ErrorKind::Interrupted
        ),
    }
}

fn lock_host(host: &Arc<Mutex<AgentHost>>) -> MutexGuard<'_, AgentHost> {
    match host.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn write_result_line(stream: &mut TcpStream, result: &HostToolResult) -> io::Result<()> {
    let mut line = serde_json::to_vec(result).map_err(io::Error::other)?;
    line.push(b'\n');
    stream.write_all(&line)?;
    stream.flush()
}

fn write_stream_headers(stream: &mut TcpStream, cursor: u64) -> io::Result<()> {
    let headers = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {NDJSON_CONTENT_TYPE}\r\nCache-Control: no-store\r\nConnection: close\r\nX-UOR-Host-Cursor: {cursor}\r\n\r\n"
    );
    stream.write_all(headers.as_bytes())?;
    stream.flush()
}

fn write_json_error(
    stream: &mut TcpStream,
    status: u16,
    code: &str,
    message: &str,
) -> Result<(), String> {
    let body = serde_json::to_vec(&json!({"code": code, "message": message}))
        .map_err(|error| error.to_string())?;
    write_http_response(
        stream,
        status,
        "application/json; charset=utf-8",
        &[("Cache-Control", "no-store")],
        &body,
    )
    .map_err(|error| error.to_string())
}

fn write_http_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    extra_headers: &[(&str, &str)],
    body: &[u8],
) -> io::Result<()> {
    let mut head = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n",
        status_text(status),
        body.len()
    );
    for (name, value) in extra_headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

fn status_text(status: u16) -> &'static str {
    match status {
        200 => "OK",
        202 => "Accepted",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        410 => "Gone",
        411 => "Length Required",
        413 => "Payload Too Large",
        415 => "Unsupported Media Type",
        500 => "Internal Server Error",
        _ => "Error",
    }
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
