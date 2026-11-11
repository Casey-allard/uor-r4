//! Loopback host intent/result transport on the existing Studio listener.
//!
//! These tests drive the real `TcpListener` and a real [`AgentHost`]; no
//! injected HTTP handler stands in for the listener, the token, the Origin or
//! the NDJSON framing. The only non-production input is the ephemeral port.

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use uor_r4_desktop_client::agent_host::{
    AgentHost, UnavailableMcpDispatch, HOST_RESULT_SCHEMA, MAX_HOST_ARGUMENT_BYTES,
    MAX_HOST_RESULTS, MAX_HOST_SUBSCRIBERS,
};
use uor_r4_desktop_client::agent_server::{StudioServer, HOST_INTENTS_PATH, HOST_RESULTS_PATH};
use uor_r4_desktop_client::{generate_launch_token, sha256_bytes, DesktopBackend, PinRegistry};
use uor_r4_studio_agent::canonical_json;

const TASK_ID: &str = "tsk_0123456789abcdef";
const RUN_ID: &str = "run_0123456789abcdef";
const HOST_INTENT_SCHEMA: &str = "uor-r4.studio.host-intent/1";

fn unique_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "uor-task18-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).expect("create fixture root");
    fs::write(dir.join("src/a.txt"), "hello").expect("write fixture");
    dir
}

fn digest(arguments: &Value) -> String {
    let canonical = canonical_json(arguments).expect("canonical arguments");
    format!("sha256:{}", sha256_bytes(canonical.as_bytes()))
}

fn data_intent(intent_id: &str, tool: &str, arguments: Value) -> Value {
    json!({
        "schema": HOST_INTENT_SCHEMA,
        "intent_id": intent_id,
        "task_id": "tsk_0000000000000000",
        "run_id": "run_0000000000000000",
        "generation": 1,
        "tool_call_id": format!("tc_{intent_id}"),
        "tool": tool,
        "request_scope": "data",
        "arguments": arguments,
        "arguments_digest": digest(&arguments),
    })
}

fn task_intent(intent_id: &str, tool: &str, arguments: Value) -> Value {
    json!({
        "schema": HOST_INTENT_SCHEMA,
        "intent_id": intent_id,
        "task_id": TASK_ID,
        "run_id": RUN_ID,
        "generation": 1,
        "tool_call_id": format!("tc_{intent_id}"),
        "tool": tool,
        "request_scope": "task",
        "arguments": arguments,
        "arguments_digest": digest(&arguments),
    })
}

struct Lab {
    host: Arc<Mutex<AgentHost>>,
    root: PathBuf,
    base: String,
    token: String,
    shutdown: Arc<AtomicBool>,
    listener_thread: Option<thread::JoinHandle<()>>,
}

impl Lab {
    fn start(name: &str, setup: impl FnOnce(&mut AgentHost, &Path)) -> Self {
        let root = unique_dir(name);
        let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
        backend
            .native_grant_folder(&root)
            .expect("grant fixture root");
        let mut host = AgentHost::new(backend).expect("secure host");
        setup(&mut host, &root);
        let host = Arc::new(Mutex::new(host));
        let token = generate_launch_token();
        let studio_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../web/studio");
        let server =
            StudioServer::start("127.0.0.1:0", studio_root, Arc::clone(&host), token.clone())
                .expect("start the Studio listener");
        let base = server.bound_host().to_string();
        let shutdown = server.shutdown_flag();
        let listener_thread = thread::spawn(move || server.serve());
        Self {
            host,
            root,
            base,
            token,
            shutdown,
            listener_thread: Some(listener_thread),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.base)
    }
}

impl Drop for Lab {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(handle) = self.listener_thread.take() {
            let _ = handle.join();
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct HttpResponse {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl HttpResponse {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap_or_else(|error| {
            panic!(
                "response body is not JSON ({error}): {}",
                String::from_utf8_lossy(&self.body)
            )
        })
    }
}

struct Conn {
    stream: TcpStream,
    buffered: Vec<u8>,
}

impl Conn {
    fn connect(base: &str) -> Self {
        let stream = TcpStream::connect(base).expect("connect to the Studio listener");
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .expect("set read timeout");
        Self {
            stream,
            buffered: Vec::new(),
        }
    }

    fn send(&mut self, request: &str) {
        self.stream
            .write_all(request.as_bytes())
            .expect("write request");
        self.stream.flush().expect("flush request");
    }

    fn send_bytes(&mut self, bytes: &[u8]) {
        self.stream.write_all(bytes).expect("write request body");
        self.stream.flush().expect("flush request body");
    }

    fn read_headers(&mut self, timeout: Duration) -> (u16, Vec<(String, String)>) {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(position) = self.buffered.windows(4).position(|w| w == b"\r\n\r\n") {
                let head: Vec<u8> = self.buffered.drain(..position + 4).collect();
                let text = String::from_utf8_lossy(&head);
                let mut lines = text.lines();
                let status = lines
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .and_then(|code| code.parse::<u16>().ok())
                    .unwrap_or(0);
                let headers = lines
                    .filter_map(|line| line.split_once(':'))
                    .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
                    .collect();
                return (status, headers);
            }
            if Instant::now() >= deadline {
                panic!(
                    "timed out reading response headers; received {:?}",
                    String::from_utf8_lossy(&self.buffered)
                );
            }
            self.stream
                .set_read_timeout(Some(deadline.saturating_duration_since(Instant::now())))
                .expect("set header timeout");
            let mut chunk = [0u8; 4096];
            match self.stream.read(&mut chunk) {
                Ok(0) => panic!("connection closed before response headers"),
                Ok(read) => self.buffered.extend_from_slice(&chunk[..read]),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(error) if error.kind() == std::io::ErrorKind::TimedOut => continue,
                Err(error) => panic!("read response headers: {error}"),
            }
        }
    }

    fn read_body(&mut self, content_length: usize) -> Vec<u8> {
        while self.buffered.len() < content_length {
            let mut chunk = [0u8; 4096];
            match self.stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(read) => self.buffered.extend_from_slice(&chunk[..read]),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == std::io::ErrorKind::TimedOut => break,
                Err(error) => panic!("read response body: {error}"),
            }
        }
        self.buffered.drain(..).collect()
    }

    fn read_line(&mut self, timeout: Duration) -> Option<String> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(position) = self.buffered.iter().position(|byte| *byte == b'\n') {
                let line: Vec<u8> = self.buffered.drain(..=position).collect();
                let text = String::from_utf8_lossy(&line).trim().to_string();
                if text.is_empty() {
                    continue;
                }
                return Some(text);
            }
            if Instant::now() >= deadline {
                return None;
            }
            self.stream
                .set_read_timeout(Some(deadline.saturating_duration_since(Instant::now())))
                .expect("set line timeout");
            let mut chunk = [0u8; 4096];
            match self.stream.read(&mut chunk) {
                Ok(0) => return None,
                Ok(read) => self.buffered.extend_from_slice(&chunk[..read]),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(error) if error.kind() == std::io::ErrorKind::TimedOut => continue,
                Err(error) => panic!("read stream line: {error}"),
            }
        }
    }

    fn closed_within(&mut self, timeout: Duration) -> bool {
        if !self.buffered.is_empty() {
            return false;
        }
        self.stream
            .set_read_timeout(Some(timeout))
            .expect("set close timeout");
        let mut byte = [0u8; 1];
        match self.stream.read(&mut byte) {
            Ok(0) => true,
            Ok(_) => false,
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                false
            }
            Err(_) => true,
        }
    }
}

fn request(
    lab: &Lab,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> HttpResponse {
    let mut conn = Conn::connect(&lab.base);
    let mut raw = format!("{method} {path} HTTP/1.1\r\nHost: {}\r\n", lab.base);
    for (name, value) in headers {
        raw.push_str(&format!("{name}: {value}\r\n"));
    }
    if !body.is_empty() {
        raw.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    raw.push_str("Connection: close\r\n\r\n");
    conn.send(&raw);
    if !body.is_empty() {
        conn.send_bytes(body);
    }
    let (status, headers) = conn.read_headers(Duration::from_secs(3));
    let length = headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.parse::<usize>().ok())
        .unwrap_or(0);
    let body = if length == 0 {
        Vec::new()
    } else {
        conn.read_body(length)
    };
    HttpResponse {
        status,
        headers,
        body,
    }
}

fn get(lab: &Lab, path: &str, origin: Option<&str>) -> HttpResponse {
    let mut headers = vec![
        ("X-UOR-Desktop-Token", lab.token.as_str()),
        ("Accept", "application/x-ndjson"),
    ];
    if let Some(origin) = origin {
        headers.push(("Origin", origin));
    }
    request(lab, "GET", path, &headers, &[])
}

fn post_intent(lab: &Lab, intent: &Value, content_type: Option<&str>) -> HttpResponse {
    let mut headers = vec![
        ("Origin", lab.url("").trim_end_matches('/').to_string()),
        ("X-UOR-Desktop-Token", lab.token.clone()),
    ];
    if let Some(content_type) = content_type {
        headers.push(("Content-Type", content_type.to_string()));
    }
    let header_refs: Vec<(&str, &str)> = headers
        .iter()
        .map(|(name, value)| (*name, value.as_str()))
        .collect();
    let body = serde_json::to_vec(intent).expect("serialize intent");
    request(lab, "POST", HOST_INTENTS_PATH, &header_refs, &body)
}

fn open_results(lab: &Lab, query: &str, origin: Option<&str>) -> Conn {
    open_results_with_headers(lab, query, origin).0
}

fn open_results_with_headers(
    lab: &Lab,
    query: &str,
    origin: Option<&str>,
) -> (Conn, Vec<(String, String)>) {
    let mut conn = Conn::connect(&lab.base);
    let mut raw = format!(
        "GET {HOST_RESULTS_PATH}{query} HTTP/1.1\r\nHost: {}\r\nX-UOR-Desktop-Token: {}\r\nAccept: application/x-ndjson\r\n",
        lab.base, lab.token
    );
    if let Some(origin) = origin {
        raw.push_str(&format!("Origin: {origin}\r\n"));
    }
    raw.push_str("Connection: close\r\n\r\n");
    conn.send(&raw);
    let (status, headers) = conn.read_headers(Duration::from_secs(2));
    assert_eq!(status, 200, "results stream status");
    let content_type = headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("content-type"))
        .map(|(_, value)| value.clone())
        .unwrap_or_default();
    assert!(
        content_type.starts_with("application/x-ndjson"),
        "results stream content type: {content_type}"
    );
    (conn, headers)
}

fn read_result(conn: &mut Conn, timeout: Duration) -> Value {
    let line = conn
        .read_line(timeout)
        .unwrap_or_else(|| panic!("no NDJSON result within {timeout:?}"));
    let value: Value = serde_json::from_str(&line).expect("NDJSON result is JSON");
    assert_eq!(
        value["schema"], HOST_RESULT_SCHEMA,
        "stream carries host results only: {value}"
    );
    assert!(value.get("type").is_none(), "no AgentEvent fields: {value}");
    assert!(
        value.get("event_id").is_none(),
        "no AgentEvent fields: {value}"
    );
    value
}

fn invoke_direct(lab: &Lab, intent: Value) -> Value {
    let mut host = lab.host.lock().expect("host lock");
    let result = host
        .invoke(&UnavailableMcpDispatch, intent)
        .expect("direct host invocation");
    serde_json::to_value(result).expect("serialize host result")
}

#[test]
fn loopback_stream_requires_token_header() {
    let lab = Lab::start("token", |_, _| {});
    let no_token = request(&lab, "GET", HOST_RESULTS_PATH, &[], &[]);
    assert_eq!(no_token.status, 403, "missing token must be refused");
    let wrong_token = request(
        &lab,
        "GET",
        HOST_RESULTS_PATH,
        &[("X-UOR-Desktop-Token", "not-the-token")],
        &[],
    );
    assert_eq!(wrong_token.status, 403, "wrong token must be refused");
    let query_token = request(
        &lab,
        "GET",
        &format!("{HOST_RESULTS_PATH}?token={}", lab.token),
        &[],
        &[],
    );
    assert_eq!(query_token.status, 403, "a URL token is never accepted");
    let mut conn = open_results(&lab, "", None);
    assert!(conn.read_line(Duration::from_millis(50)).is_none());
}

#[test]
fn loopback_stream_requires_exact_origin() {
    let lab = Lab::start("origin", |_, _| {});
    let foreign = get(&lab, HOST_RESULTS_PATH, Some("https://example.invalid"));
    assert_eq!(foreign.status, 403, "foreign Origin must be refused");
    let other_loopback = get(&lab, HOST_RESULTS_PATH, Some("http://127.0.0.1:1"));
    assert_eq!(
        other_loopback.status, 403,
        "a different loopback origin is not the bound origin"
    );
    let exact = lab.url("");
    let mut accepted = open_results(&lab, "", Some(exact.trim_end_matches('/')));
    assert!(accepted.read_line(Duration::from_millis(50)).is_none());
    let mut absent = open_results(&lab, "", None);
    assert!(absent.read_line(Duration::from_millis(50)).is_none());
}

#[test]
fn loopback_stream_is_ndjson_not_event_source() {
    let lab = Lab::start("ndjson", |_, _| {});
    let mut stream = open_results(&lab, "", None);
    let intent = data_intent("hti_data_000000000001", "file.tree", json!({"path": "src"}));
    let started = Instant::now();
    let response = post_intent(&lab, &intent, Some("application/json"));
    assert_eq!(response.status, 202, "intent POST is accepted");
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "POST is not held by the open stream"
    );
    let result = read_result(&mut stream, Duration::from_secs(3));
    assert_eq!(result["intent_id"], intent["intent_id"]);
    assert_eq!(result["request_scope"], "data");

    let event_stream = request(
        &lab,
        "GET",
        "/v1/studio/host/events",
        &[("X-UOR-Desktop-Token", lab.token.as_str())],
        &[],
    );
    assert_eq!(event_stream.status, 404, "no SSE/EventSource route exists");
    let accept_sse = request(
        &lab,
        "GET",
        HOST_RESULTS_PATH,
        &[
            ("X-UOR-Desktop-Token", lab.token.as_str()),
            ("Accept", "text/event-stream"),
        ],
        &[],
    );
    let content_type = accept_sse
        .header("content-type")
        .unwrap_or_default()
        .to_string();
    assert!(
        content_type.starts_with("application/x-ndjson"),
        "Accept: text/event-stream does not flip the route to SSE"
    );
}

#[test]
fn loopback_stream_replays_from_cursor() {
    let lab = Lab::start("replay", |_, _| {});
    let mut first_stream = open_results(&lab, "", None);
    for index in 0..2 {
        let intent = data_intent(
            &format!("hti_data_00000000001{index}"),
            "file.tree",
            json!({"path": "src"}),
        );
        let response = post_intent(&lab, &intent, Some("application/json"));
        assert_eq!(response.status, 202);
    }
    let first = read_result(&mut first_stream, Duration::from_secs(3));
    let second = read_result(&mut first_stream, Duration::from_secs(3));
    let first_cursor = first["cursor"].as_str().expect("first cursor");
    let second_cursor = second["cursor"].as_str().expect("second cursor");
    assert!(second_cursor > first_cursor);
    drop(first_stream);

    let mut replay = open_results(&lab, &format!("?cursor={first_cursor}"), None);
    let replayed = read_result(&mut replay, Duration::from_secs(3));
    assert_eq!(replayed["cursor"], second["cursor"], "exact replay");
    assert_eq!(
        replayed["intent_id"], second["intent_id"],
        "no earlier frames"
    );
    assert!(
        replay.read_line(Duration::from_millis(300)).is_none(),
        "replay is bounded to the retained result set"
    );
}

#[test]
fn loopback_stream_returns_410_for_expired_cursor() {
    let lab = Lab::start("expired", |_, _| {});
    for index in 0..=MAX_HOST_RESULTS {
        let intent = data_intent(
            &format!("hti_data_{index:012}"),
            "file.tree",
            json!({"path": "src"}),
        );
        let result = invoke_direct(&lab, intent);
        assert_eq!(result["status"], "ok");
    }
    let expired = request(
        &lab,
        "GET",
        &format!("{HOST_RESULTS_PATH}?cursor=0"),
        &[("X-UOR-Desktop-Token", lab.token.as_str())],
        &[],
    );
    assert_eq!(expired.status, 410, "dropped cursor is gone");
    assert_eq!(expired.json()["code"], "cursor_expired");
    let latest = MAX_HOST_RESULTS as u64 + 1;
    let mut retained = open_results(&lab, &format!("?cursor={latest}"), None);
    assert!(retained.read_line(Duration::from_millis(150)).is_none());
}

#[test]
fn loopback_data_uses_existing_post_path() {
    let lab = Lab::start("data", |_, _| {});
    let mut stream = open_results(&lab, "", None);
    let intent = data_intent("hti_data_000000000021", "file.tree", json!({"path": "src"}));
    let response = post_intent(&lab, &intent, Some("application/json"));
    assert_eq!(response.status, 202);
    let accepted = response.json();
    assert_eq!(accepted["accepted"], true);
    assert_eq!(accepted["intent_id"], intent["intent_id"]);
    let result = read_result(&mut stream, Duration::from_secs(3));
    assert_eq!(result["request_scope"], "data");
    assert_eq!(result["output"]["path"], "src");
    let entries = result["output"]["entries"].as_array().expect("entries");
    assert!(entries.iter().any(|entry| entry["name"] == "a.txt"));

    let wrong_route = request(
        &lab,
        "GET",
        "/v1/studio/host/data",
        &[("X-UOR-Desktop-Token", lab.token.as_str())],
        &[],
    );
    assert_eq!(
        wrong_route.status, 404,
        "no second data route is registered"
    );
    let wrong_method = request(
        &lab,
        "POST",
        HOST_RESULTS_PATH,
        &[
            ("Origin", lab.url("").trim_end_matches('/')),
            ("Content-Type", "application/json"),
            ("X-UOR-Desktop-Token", &lab.token),
        ],
        b"{}",
    );
    assert_eq!(wrong_method.status, 405, "results route is GET-only");
}

#[test]
fn loopback_intent_post_requires_json_content_type() {
    let lab = Lab::start("content-type", |_, _| {});
    let intent = data_intent("hti_data_000000000031", "file.tree", json!({"path": "src"}));
    let missing = post_intent(&lab, &intent, None);
    assert_eq!(missing.status, 415);
    let text = post_intent(&lab, &intent, Some("text/plain"));
    assert_eq!(text.status, 415);
}

#[test]
fn loopback_intent_post_reads_body_beyond_one_socket_read() {
    let lab = Lab::start("large-body", |_, _| {});
    let padding = "x".repeat(100_000);
    let intent = data_intent(
        "hti_data_000000000041",
        "file.tree",
        json!({"path": "src", "padding": padding}),
    );
    let mut stream = open_results(&lab, "", None);
    let response = post_intent(&lab, &intent, Some("application/json"));
    assert_eq!(response.status, 202, "the whole declared body was read");
    let result = read_result(&mut stream, Duration::from_secs(3));
    assert_eq!(result["status"], "ok");
    assert_eq!(result["intent_id"], intent["intent_id"]);
}

#[test]
fn no_remote_tls_route_is_registered() {
    let root = unique_dir("tls");
    let host = Arc::new(Mutex::new(
        AgentHost::new(DesktopBackend::new(PinRegistry::with_synthetic_standin()))
            .expect("secure host"),
    ));
    let studio_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../web/studio");
    for bind in ["0.0.0.0:0", "[::]:0", "192.0.2.10:0"] {
        let refused = StudioServer::start(bind, &studio_root, Arc::clone(&host), "t".into());
        assert!(refused.is_err(), "non-loopback bind {bind} must be refused");
    }
    let lab = Lab::start("tls-live", |_, _| {});
    assert!(lab.base.starts_with("127.0.0.1:"));
    let proxied = get(&lab, HOST_RESULTS_PATH, None);
    assert_eq!(proxied.status, 200, "the route is plain loopback HTTP");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn no_second_listener_is_opened() {
    let lab = Lab::start("single-listener", |_, _| {});
    let address: std::net::SocketAddr = lab.base.parse().expect("bound address");
    assert!(
        TcpListener::bind(address).is_err(),
        "the single listener still owns the bound address"
    );
    let index = request(&lab, "GET", "/", &[], &[]);
    assert_eq!(index.status, 200);
    assert!(
        String::from_utf8_lossy(&index.body).contains("UOR_DESKTOP_SESSION_TOKEN"),
        "the static surface is on the same listener"
    );
    let invoke = request(
        &lab,
        "POST",
        "/__tauri_invoke",
        &[
            ("Origin", lab.url("").trim_end_matches('/')),
            ("Content-Type", "application/json"),
            ("X-UOR-Desktop-Token", &lab.token),
        ],
        br#"{"id":1,"command":"native_list_granted_folders","args":{}}"#,
    );
    assert_eq!(invoke.status, 200);
    assert_eq!(invoke.json()["ok"], true);
    let mut stream = open_results(&lab, "", None);
    let response = post_intent(
        &lab,
        &data_intent("hti_data_000000000051", "file.tree", json!({"path": "src"})),
        Some("application/json"),
    );
    assert_eq!(response.status, 202);
    assert_eq!(
        read_result(&mut stream, Duration::from_secs(3))["status"],
        "ok"
    );
}

#[test]
fn held_ndjson_stream_does_not_block_intent_post() {
    let lab = Lab::start("held", |_, _| {});
    let mut stream = open_results(&lab, "", None);
    let started = Instant::now();
    let response = post_intent(
        &lab,
        &data_intent("hti_data_000000000061", "file.tree", json!({"path": "src"})),
        Some("application/json"),
    );
    let elapsed = started.elapsed();
    assert_eq!(response.status, 202);
    assert!(
        elapsed < Duration::from_millis(1_500),
        "a held stream must not block intent POST; took {elapsed:?}"
    );
    assert_eq!(
        read_result(&mut stream, Duration::from_secs(3))["status"],
        "ok"
    );
}

#[test]
fn loopback_accepts_intent_before_slow_host_execution_and_settles_once() {
    let lab = Lab::start("slow", |_, _| {});
    let mut stream = open_results(&lab, "", None);
    let held = lab.host.lock().expect("hold the host mutex");
    let started = Instant::now();
    let response = post_intent(
        &lab,
        &data_intent("hti_data_000000000071", "file.tree", json!({"path": "src"})),
        Some("application/json"),
    );
    assert_eq!(response.status, 202);
    assert!(
        started.elapsed() < Duration::from_millis(1_000),
        "POST returns before host execution completes"
    );
    drop(held);
    let result = read_result(&mut stream, Duration::from_secs(3));
    assert_eq!(result["status"], "ok");
    assert!(
        stream.read_line(Duration::from_millis(300)).is_none(),
        "a slow host call produces exactly one result"
    );
}

#[test]
fn loopback_capabilities_report_no_native_approval_surface() {
    let lab = Lab::start("capabilities", |_, _| {});
    let response = request(
        &lab,
        "POST",
        "/__tauri_invoke",
        &[
            ("Origin", lab.url("").trim_end_matches('/')),
            ("Content-Type", "application/json"),
            ("X-UOR-Desktop-Token", &lab.token),
        ],
        br#"{"id":1,"command":"agent_host_capabilities","args":{}}"#,
    );
    assert_eq!(response.status, 200);
    let body = response.json();
    assert_eq!(body["ok"], true);
    assert_eq!(
        body["result"]["approvals"], false,
        "the dialog-less loopback surface must not advertise approvals"
    );
}

#[test]
fn loopback_approval_required_tool_is_deny_only() {
    let lab = Lab::start("approvals", |_, _| {});
    let mut stream = open_results(&lab, "", None);
    let intent = task_intent(
        "hti_0123456789abcdef_000000000081",
        "file.read",
        json!({"path": "src/a.txt"}),
    );
    let response = post_intent(&lab, &intent, Some("application/json"));
    assert_eq!(response.status, 202);
    let result = read_result(&mut stream, Duration::from_secs(3));
    assert_eq!(result["status"], "error");
    assert_eq!(result["code"], "approval_forbidden");
    assert_eq!(result["output"]["approval_required"], true);
    assert_eq!(result["output"]["approved"], false);
    assert!(result["output"].get("content").is_none());
    let approval_id = result["output"]["approval_id"]
        .as_str()
        .expect("host-issued approval identity")
        .to_string();
    assert!(approval_id.starts_with("apr_"));

    let approve = request(
        &lab,
        "POST",
        "/__tauri_invoke",
        &[
            ("Origin", lab.url("").trim_end_matches('/')),
            ("Content-Type", "application/json"),
            ("X-UOR-Desktop-Token", &lab.token),
        ],
        format!(
            "{{\"id\":2,\"command\":\"agent_host_resolve_approval\",\"args\":{{\"input\":{{\"approval_id\":\"{approval_id}\",\"decision\":\"approved\"}}}}}}"
        )
        .as_bytes(),
    );
    let body = approve.json();
    assert_eq!(body["ok"], false, "page approval cannot self-confirm");
    assert!(
        body["error"]
            .as_str()
            .unwrap_or_default()
            .contains("host confirmation is required"),
        "approval requires a native confirmation surface"
    );

    let deny = request(
        &lab,
        "POST",
        "/__tauri_invoke",
        &[
            ("Origin", lab.url("").trim_end_matches('/')),
            ("Content-Type", "application/json"),
            ("X-UOR-Desktop-Token", &lab.token),
        ],
        format!(
            "{{\"id\":3,\"command\":\"agent_host_resolve_approval\",\"args\":{{\"input\":{{\"approval_id\":\"{approval_id}\",\"decision\":\"denied\"}}}}}}"
        )
        .as_bytes(),
    );
    assert_eq!(deny.json()["ok"], true, "denial is the allowed decision");
}

#[test]
fn loopback_artifact_result_preserves_full64_identity_and_range_metadata() {
    let lab = Lab::start("artifact", |host, root| {
        fs::write(root.join("data.bin"), b"0123456789").expect("write artifact");
        let grant = host.backend().agent_grants()[0].grant_id.clone();
        host.backend_mut()
            .agent_register_artifact(&grant, "data.bin", "application/octet-stream")
            .expect("register artifact");
    });
    let artifact_id = format!("art_{}", sha256_bytes(b"0123456789"));
    assert_eq!(artifact_id.len(), 4 + 64, "full64 artifact identity");
    assert!(artifact_id.starts_with("art_"));
    let mut stream = open_results(&lab, "", None);
    let intent = data_intent(
        "hti_data_000000000091",
        "artifact.read",
        json!({"artifact_id": artifact_id, "start": 2, "end": 5}),
    );
    assert_eq!(
        post_intent(&lab, &intent, Some("application/json")).status,
        202
    );
    let result = read_result(&mut stream, Duration::from_secs(3));
    let output = &result["output"];
    assert_eq!(output["artifact_id"], artifact_id);
    assert_eq!(output["size_bytes"], 3);
    assert_eq!(output["artifact_size_bytes"], 10);
    assert_eq!(output["range"], json!({"start": 2, "end": 5}));
    assert_eq!(output["bytes_base64"], "MjM0");
    assert_eq!(
        output["whole_digest"],
        format!("sha256:{}", sha256_bytes(b"0123456789"))
    );
    assert_eq!(output["digest"], format!("sha256:{}", sha256_bytes(b"234")));
}

#[test]
fn loopback_rejects_duplicate_authority_headers() {
    let lab = Lab::start("duplicate-headers", |_, _| {});

    let mut duplicate_host = Conn::connect(&lab.base);
    duplicate_host.send(&format!(
        "GET {HOST_RESULTS_PATH} HTTP/1.1\r\nHost: {0}\r\nHost: evil.invalid\r\nX-UOR-Desktop-Token: {1}\r\nConnection: close\r\n\r\n",
        lab.base, lab.token
    ));
    assert_eq!(duplicate_host.read_headers(Duration::from_secs(2)).0, 403);

    let mut duplicate_token = Conn::connect(&lab.base);
    duplicate_token.send(&format!(
        "GET {HOST_RESULTS_PATH} HTTP/1.1\r\nHost: {0}\r\nX-UOR-Desktop-Token: {1}\r\nX-UOR-Desktop-Token: {1}\r\nConnection: close\r\n\r\n",
        lab.base, lab.token
    ));
    assert_eq!(duplicate_token.read_headers(Duration::from_secs(2)).0, 403);

    let intent = data_intent("hti_data_000000000111", "file.tree", json!({"path": "src"}));
    let body = serde_json::to_vec(&intent).expect("intent body");
    let mut duplicate_length = Conn::connect(&lab.base);
    duplicate_length.send(&format!(
        "POST {HOST_INTENTS_PATH} HTTP/1.1\r\nHost: {0}\r\nOrigin: http://{0}\r\nContent-Type: application/json\r\nX-UOR-Desktop-Token: {1}\r\nContent-Length: {2}\r\nContent-Length: {2}\r\nConnection: close\r\n\r\n",
        lab.base,
        lab.token,
        body.len()
    ));
    duplicate_length.send_bytes(&body);
    assert_eq!(duplicate_length.read_headers(Duration::from_secs(2)).0, 400);
}

#[test]
fn loopback_rejects_non_identity_transfer_encoding() {
    let lab = Lab::start("te-duplicate", |_, _| {});
    let intent = data_intent("hti_data_000000000121", "file.tree", json!({"path": "src"}));
    let body = serde_json::to_vec(&intent).expect("intent body");
    let mut conn = Conn::connect(&lab.base);
    conn.send(&format!(
        "POST {HOST_INTENTS_PATH} HTTP/1.1\r\nHost: {0}\r\nOrigin: http://{0}\r\nContent-Type: application/json\r\nX-UOR-Desktop-Token: {1}\r\nTransfer-Encoding: identity\r\nTransfer-Encoding: chunked\r\nContent-Length: {2}\r\nConnection: close\r\n\r\n",
        lab.base,
        lab.token,
        body.len()
    ));
    conn.send_bytes(&body);
    assert_eq!(
        conn.read_headers(Duration::from_secs(2)).0,
        411,
        "every Transfer-Encoding header must be identity"
    );
}

#[test]
fn loopback_intent_framing_is_bounded() {
    let lab = Lab::start("framing", |_, _| {});

    let mut chunked = Conn::connect(&lab.base);
    chunked.send(&format!(
        "POST {HOST_INTENTS_PATH} HTTP/1.1\r\nHost: {0}\r\nOrigin: http://{0}\r\nContent-Type: application/json\r\nX-UOR-Desktop-Token: {1}\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
        lab.base, lab.token
    ));
    assert_eq!(chunked.read_headers(Duration::from_secs(2)).0, 411);

    let mut oversized = Conn::connect(&lab.base);
    oversized.send(&format!(
        "POST {HOST_INTENTS_PATH} HTTP/1.1\r\nHost: {0}\r\nOrigin: http://{0}\r\nContent-Type: application/json\r\nX-UOR-Desktop-Token: {1}\r\nContent-Length: {2}\r\nConnection: close\r\n\r\n",
        lab.base,
        lab.token,
        MAX_HOST_ARGUMENT_BYTES + 1_000_000
    ));
    assert_eq!(oversized.read_headers(Duration::from_secs(2)).0, 413);
}

#[test]
fn loopback_authorizes_host_routes_before_reading_bodies() {
    let lab = Lab::start("pre-auth", |_, _| {});
    let huge = 19 * 1024 * 1024;

    let mut intent = Conn::connect(&lab.base);
    intent.send(&format!(
        "POST {HOST_INTENTS_PATH} HTTP/1.1\r\nHost: {0}\r\nOrigin: http://{0}\r\nContent-Type: application/json\r\nContent-Length: {huge}\r\nConnection: close\r\n\r\n",
        lab.base
    ));
    assert_eq!(
        intent.read_headers(Duration::from_millis(1_500)).0,
        403,
        "an unauthenticated intent must be refused before its body is read"
    );

    let mut results = Conn::connect(&lab.base);
    results.send(&format!(
        "GET {HOST_RESULTS_PATH} HTTP/1.1\r\nHost: {0}\r\nContent-Length: {huge}\r\nConnection: close\r\n\r\n",
        lab.base
    ));
    assert_eq!(
        results.read_headers(Duration::from_millis(1_500)).0,
        403,
        "an unauthenticated result request must be refused before its body is read"
    );
}

#[test]
fn loopback_stream_resume_after_mid_backlog_disconnect() {
    let lab = Lab::start("resume", |_, _| {});
    for index in 0..2 {
        let result = invoke_direct(
            &lab,
            data_intent(
                &format!("hti_data_00000000013{index}"),
                "file.tree",
                json!({"path": "src"}),
            ),
        );
        assert_eq!(result["status"], "ok");
    }

    let (mut first, headers) = open_results_with_headers(&lab, "?cursor=0", None);
    let header_cursor = headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("x-uor-host-cursor"))
        .map(|(_, value)| value.as_str())
        .unwrap_or_default();
    assert_eq!(
        header_cursor, "0",
        "the stream must not advertise a cursor beyond unprocessed replay"
    );
    let first_result = read_result(&mut first, Duration::from_secs(3));
    assert_eq!(first_result["cursor"], "1");
    drop(first);

    let (mut second, headers) = open_results_with_headers(&lab, "?cursor=1", None);
    let header_cursor = headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("x-uor-host-cursor"))
        .map(|(_, value)| value.as_str())
        .unwrap_or_default();
    assert_eq!(header_cursor, "1");
    let resumed = read_result(&mut second, Duration::from_secs(3));
    assert_eq!(resumed["cursor"], "2");
    assert_eq!(
        resumed["intent_id"], "hti_data_000000000131",
        "the resumed stream replays the unprocessed result exactly once"
    );
    assert!(second.read_line(Duration::from_millis(300)).is_none());
}

#[test]
fn loopback_rejects_wrong_or_missing_host() {
    let lab = Lab::start("host-guard", |_, _| {});

    let mut wrong = Conn::connect(&lab.base);
    wrong.send(&format!(
        "GET {HOST_RESULTS_PATH} HTTP/1.1\r\nHost: evil.invalid\r\nX-UOR-Desktop-Token: {}\r\nConnection: close\r\n\r\n",
        lab.token
    ));
    assert_eq!(wrong.read_headers(Duration::from_secs(2)).0, 403);

    let mut missing = Conn::connect(&lab.base);
    missing.send(&format!(
        "GET {HOST_RESULTS_PATH} HTTP/1.1\r\nX-UOR-Desktop-Token: {}\r\nConnection: close\r\n\r\n",
        lab.token
    ));
    assert_eq!(missing.read_headers(Duration::from_secs(2)).0, 403);
}

#[test]
fn loopback_rejects_url_token_even_with_a_valid_header() {
    let lab = Lab::start("url-token", |_, _| {});
    let response = get(
        &lab,
        &format!("{HOST_RESULTS_PATH}?cursor=0&token={}", lab.token),
        None,
    );
    assert_eq!(response.status, 403, "the token is header-only");
}

#[test]
fn loopback_rejects_duplicate_origin() {
    let lab = Lab::start("origin-duplicate", |_, _| {});
    let origin = lab.url("");
    let mut conn = Conn::connect(&lab.base);
    conn.send(&format!(
        "GET {HOST_RESULTS_PATH} HTTP/1.1\r\nHost: {0}\r\nX-UOR-Desktop-Token: {1}\r\nOrigin: {2}\r\nOrigin: {2}\r\nConnection: close\r\n\r\n",
        lab.base,
        lab.token,
        origin.trim_end_matches('/')
    ));
    assert_eq!(conn.read_headers(Duration::from_secs(2)).0, 403);
}

#[test]
fn evicted_result_stream_closes_without_hanging() {
    let lab = Lab::start("eviction", |_, _| {});
    let mut streams: Vec<Conn> = Vec::new();
    for _ in 0..(MAX_HOST_SUBSCRIBERS + 1) {
        streams.push(open_results(&lab, "", None));
    }
    assert!(
        streams[0].closed_within(Duration::from_secs(2)),
        "the evicted stream must terminate, not wait indefinitely"
    );
    assert!(
        !streams[MAX_HOST_SUBSCRIBERS].closed_within(Duration::from_millis(200)),
        "the newest subscriber remains open"
    );
}
