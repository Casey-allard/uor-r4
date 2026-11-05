use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(test)]
use std::sync::Barrier;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[cfg(test)]
static REGISTRATION_PROBE: Mutex<Option<(Arc<Barrier>, Arc<Barrier>)>> = Mutex::new(None);

use serde::{Deserialize, Serialize};

const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const MAX_HEADER_BYTES: usize = 64 * 1024;
const STARTUP_TIMEOUT: Duration = Duration::from_secs(10);
const IO_TIMEOUT: Duration = Duration::from_millis(100);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NativeProxyResponse {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NativeChatEvent {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body_chunk: Vec<u8>,
    pub terminal: bool,
    pub error: Option<String>,
}

struct SidecarChild {
    child: Mutex<Child>,
}

impl Drop for SidecarChild {
    fn drop(&mut self) {
        if let Ok(mut child) = self.child.lock() {
            super::process_control::terminate(&mut child);
        }
    }
}

struct StartupChild(Option<Child>);

impl Drop for StartupChild {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            super::process_control::terminate(&mut child);
        }
    }
}

#[derive(Clone)]
pub struct SidecarProcess {
    port: u16,
    token: String,
    child: Arc<SidecarChild>,
    active: Arc<Mutex<BTreeMap<String, Arc<AtomicBool>>>>,
}

impl std::fmt::Debug for SidecarProcess {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SidecarProcess")
            .field("port", &self.port)
            .finish_non_exhaustive()
    }
}

impl SidecarProcess {
    pub fn start(binary: &Path, stack: &Path, tokenizer: &Path) -> Result<Self, String> {
        for path in [binary, stack, tokenizer] {
            if !path.is_file() {
                return Err(format!(
                    "required sidecar input is unavailable: {}",
                    path.display()
                ));
            }
        }
        let token = generate_token();
        let mut command = Command::new(binary);
        super::process_control::prepare(&mut command);
        let child = command
            .arg("--stack")
            .arg(stack)
            .arg("--tokenizer")
            .arg(tokenizer)
            .arg("--serve")
            .arg("127.0.0.1:0")
            .arg("--token-stdin")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("could not launch uor-chat: {error}"))?;
        let mut startup = StartupChild(Some(child));
        let child = startup
            .0
            .as_mut()
            .ok_or_else(|| "uor-chat child was unavailable".to_string())?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| "uor-chat stdin was unavailable".to_string())?;
        writeln!(stdin, "{token}")
            .map_err(|error| format!("could not pass token to uor-chat: {error}"))?;
        drop(stdin);
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "uor-chat stdout was unavailable".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "uor-chat stderr was unavailable".to_string())?;
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stderr);
            let mut sink = std::io::sink();
            let _ = std::io::copy(&mut reader, &mut sink);
        });
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if sender.send(line).is_err() {
                    break;
                }
            }
        });
        let deadline = Instant::now() + STARTUP_TIMEOUT;
        let mut port = None;
        while port.is_none() {
            if Instant::now() >= deadline {
                return Err("uor-chat did not report its loopback port before timeout".into());
            }
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(line) => {
                    if let Some(address) =
                        line.strip_prefix("uor-chat OpenAI loopback listening on http://127.0.0.1:")
                    {
                        let parsed = address
                            .split_whitespace()
                            .next()
                            .unwrap_or_default()
                            .parse::<u16>()
                            .map_err(|_| format!("invalid uor-chat port in {line}"))?;
                        port = Some(parsed);
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    return Err("uor-chat exited before reporting its port".into());
                }
            }
        }
        let child = startup
            .0
            .take()
            .ok_or_else(|| "uor-chat child was unavailable after startup".to_string())?;
        let sidecar = Self {
            port: port.ok_or_else(|| "uor-chat did not report a port".to_string())?,
            token,
            child: Arc::new(SidecarChild {
                child: Mutex::new(child),
            }),
            active: Arc::new(Mutex::new(BTreeMap::new())),
        };
        let readiness_deadline = Instant::now() + STARTUP_TIMEOUT;
        loop {
            match sidecar.models() {
                Ok(response) if response.status == 200 => return Ok(sidecar),
                Ok(response) => {
                    if Instant::now() >= readiness_deadline {
                        sidecar.shutdown();
                        return Err(format!(
                            "uor-chat model listing returned HTTP {}",
                            response.status
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(error) => {
                    if Instant::now() >= readiness_deadline {
                        sidecar.shutdown();
                        return Err(format!("uor-chat was not ready: {error}"));
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }

    pub fn shutdown(&self) {
        if let Ok(active) = self.active.lock() {
            for flag in active.values() {
                flag.store(true, Ordering::Relaxed);
            }
        }
        if let Ok(mut child) = self.child.child.lock() {
            super::process_control::terminate(&mut child);
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn models(&self) -> Result<NativeProxyResponse, String> {
        let (reader, status, headers) =
            self.request("GET", "/v1/models", &[], "application/json")?;
        let body = read_bounded(reader, None, "")?;
        Ok(NativeProxyResponse {
            status,
            headers,
            body,
        })
    }

    pub fn decompose(
        &self,
        width: u32,
        height: u32,
        columns: u32,
        rows: u32,
        rgba: &[u8],
    ) -> Result<NativeProxyResponse, String> {
        if width == 0 || height == 0 || width > 256 || height > 256 {
            return Err("image dimensions must be within 1..256".into());
        }
        if columns == 0
            || rows == 0
            || columns > 32
            || rows > 32
            || columns > width
            || rows > height
        {
            return Err("image grid must fit within 32x32 and the image dimensions".into());
        }
        if rgba.len() != width as usize * height as usize * 4 {
            return Err("RGBA byte length does not match dimensions".into());
        }
        let path = format!(
            "/v1/images/decompose?width={width}&height={height}&columns={columns}&rows={rows}"
        );
        let (reader, status, headers) =
            self.request("POST", &path, rgba, "application/octet-stream")?;
        let body = read_bounded(reader, None, "")?;
        Ok(NativeProxyResponse {
            status,
            headers,
            body,
        })
    }

    pub fn stream_chat<F>(&self, request_id: &str, body: &str, emit: F) -> Result<(), String>
    where
        F: FnMut(NativeChatEvent),
    {
        if request_id.trim().is_empty() {
            return Err("request ID is required".into());
        }
        let cancel = self
            .active
            .lock()
            .map_err(|_| "request registry mutex poisoned".to_string())?
            .entry(request_id.to_string())
            .or_insert_with(|| Arc::new(AtomicBool::new(false)))
            .clone();
        #[cfg(test)]
        if let Some(probe) = REGISTRATION_PROBE
            .lock()
            .map_err(|_| "registration probe mutex poisoned".to_string())?
            .clone()
        {
            probe.0.wait();
            probe.1.wait();
        }
        let result = self.stream_chat_inner(request_id, body, cancel, emit);
        self.active
            .lock()
            .map_err(|_| "request registry mutex poisoned".to_string())?
            .remove(request_id);
        result
    }

    fn stream_chat_inner<F>(
        &self,
        request_id: &str,
        body: &str,
        cancel: Arc<AtomicBool>,
        mut emit: F,
    ) -> Result<(), String>
    where
        F: FnMut(NativeChatEvent),
    {
        let (reader, status, headers) = self.request(
            "POST",
            "/v1/chat/completions",
            body.as_bytes(),
            "application/json",
        )?;
        let is_sse = headers
            .get("content-type")
            .is_some_and(|value| value.to_ascii_lowercase().contains("text/event-stream"));
        if !is_sse {
            let body_chunk = read_bounded(reader, Some(cancel.clone()), request_id)?;
            emit(NativeChatEvent {
                status,
                headers,
                body_chunk,
                terminal: true,
                error: None,
            });
            return Ok(());
        }
        let mut reader = reader;
        let mut block = Vec::new();
        let mut line = Vec::new();
        let mut idle_deadline = Instant::now() + RESPONSE_IDLE_TIMEOUT;
        loop {
            if cancel.load(Ordering::Relaxed) {
                let _ = reader.get_ref().shutdown(Shutdown::Both);
                emit(NativeChatEvent {
                    status,
                    headers,
                    body_chunk: Vec::new(),
                    terminal: true,
                    error: Some(format!("request {request_id} cancelled")),
                });
                return Ok(());
            }
            match read_line_bytes(&mut reader, &mut line, MAX_RESPONSE_BYTES)? {
                LineRead::Eof => break,
                LineRead::Pending if Instant::now() < idle_deadline => continue,
                LineRead::Pending => return Err("timed out waiting for sidecar stream".into()),
                LineRead::Line => {
                    idle_deadline = Instant::now() + RESPONSE_IDLE_TIMEOUT;
                    if block.len() + line.len() > MAX_RESPONSE_BYTES {
                        return Err("sidecar stream event exceeds 1 MiB".into());
                    }
                    if line == b"\r\n" || line == b"\n" {
                        if !block.is_empty() {
                            block.extend_from_slice(&line);
                            emit(NativeChatEvent {
                                status,
                                headers: headers.clone(),
                                body_chunk: std::mem::take(&mut block),
                                terminal: false,
                                error: None,
                            });
                        }
                    } else {
                        block.extend_from_slice(&line);
                    }
                    line.clear();
                }
            }
        }
        if !block.is_empty() {
            emit(NativeChatEvent {
                status,
                headers: headers.clone(),
                body_chunk: block,
                terminal: false,
                error: None,
            });
        }
        emit(NativeChatEvent {
            status,
            headers,
            body_chunk: Vec::new(),
            terminal: true,
            error: None,
        });
        Ok(())
    }

    pub fn cancel(&self, request_id: &str) -> Result<(), String> {
        if request_id.trim().is_empty() {
            return Err("request ID is required for cancellation".into());
        }
        let mut active = self
            .active
            .lock()
            .map_err(|_| "request registry mutex poisoned".to_string())?;
        if !active.contains_key(request_id) && active.len() >= 64 {
            return Err("request cancellation registry is full".into());
        }
        active
            .entry(request_id.to_string())
            .or_insert_with(|| Arc::new(AtomicBool::new(false)))
            .store(true, Ordering::Relaxed);
        Ok(())
    }

    fn request(
        &self,
        method: &str,
        path: &str,
        body: &[u8],
        content_type: &str,
    ) -> Result<(BufReader<TcpStream>, u16, BTreeMap<String, String>), String> {
        let mut stream = TcpStream::connect(("127.0.0.1", self.port))
            .map_err(|error| format!("could not connect to uor-chat: {error}"))?;
        stream
            .set_read_timeout(Some(IO_TIMEOUT))
            .map_err(|error| error.to_string())?;
        stream
            .set_write_timeout(Some(IO_TIMEOUT))
            .map_err(|error| error.to_string())?;
        let head = format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAuthorization: Bearer {token}\r\nContent-Type: {content_type}\r\nAccept: application/json, text/event-stream, application/octet-stream\r\nContent-Length: {length}\r\nConnection: close\r\n\r\n",
            port = self.port,
            token = self.token,
            length = body.len()
        );
        stream
            .write_all(head.as_bytes())
            .and_then(|_| stream.write_all(body))
            .map_err(|error| format!("could not write sidecar request: {error}"))?;
        let mut reader = BufReader::new(stream);
        let status_line = read_header_line(&mut reader)?;
        let status = status_line
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| format!("invalid sidecar status line: {status_line}"))?
            .parse::<u16>()
            .map_err(|_| format!("invalid sidecar status: {status_line}"))?;
        let mut headers = BTreeMap::new();
        let mut consumed = status_line.len();
        loop {
            let line = read_header_line(&mut reader)?;
            consumed += line.len();
            if consumed > MAX_HEADER_BYTES {
                return Err("sidecar headers exceed 64 KiB".into());
            }
            if line == "\r\n" || line == "\n" {
                break;
            }
            if let Some((name, value)) = line.split_once(':') {
                headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
            }
        }
        Ok((reader, status, headers))
    }
}

const RESPONSE_IDLE_TIMEOUT: Duration = Duration::from_secs(15);

enum LineRead {
    Line,
    Pending,
    Eof,
}

fn read_line_bytes(
    reader: &mut impl BufRead,
    line: &mut Vec<u8>,
    maximum: usize,
) -> Result<LineRead, String> {
    loop {
        let (consumed, complete) = match reader.fill_buf() {
            Ok([]) => return Ok(LineRead::Eof),
            Ok(available) => match available.iter().position(|byte| *byte == b'\n') {
                Some(position) => {
                    let consumed = position + 1;
                    line.extend_from_slice(&available[..consumed]);
                    (consumed, true)
                }
                None => {
                    line.extend_from_slice(available);
                    (available.len(), false)
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                return Ok(LineRead::Pending)
            }
            Err(error) => return Err(format!("could not read sidecar data: {error}")),
        };
        if line.len() > maximum {
            return Err("sidecar line exceeds its byte limit".into());
        }
        reader.consume(consumed);
        if complete {
            return Ok(LineRead::Line);
        }
    }
}

fn read_header_line(reader: &mut impl BufRead) -> Result<String, String> {
    let deadline = Instant::now() + RESPONSE_IDLE_TIMEOUT;
    let mut line = Vec::new();
    loop {
        match read_line_bytes(reader, &mut line, MAX_HEADER_BYTES)? {
            LineRead::Line => {
                return String::from_utf8(line)
                    .map_err(|_| "sidecar headers are not valid UTF-8".into());
            }
            LineRead::Pending if Instant::now() < deadline => {}
            LineRead::Pending => return Err("timed out waiting for sidecar headers".into()),
            LineRead::Eof => return Err("sidecar closed before completing headers".into()),
        }
    }
}

fn read_bounded(
    mut reader: BufReader<TcpStream>,
    cancel: Option<Arc<AtomicBool>>,
    request_id: &str,
) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 8192];
    let mut deadline = Instant::now() + RESPONSE_IDLE_TIMEOUT;
    loop {
        if cancel
            .as_ref()
            .is_some_and(|flag| flag.load(Ordering::Relaxed))
        {
            let _ = reader.get_ref().shutdown(Shutdown::Both);
            return Err(format!("request {request_id} cancelled"));
        }
        match reader.read(&mut buffer) {
            Ok(0) => return Ok(bytes),
            Ok(read) => {
                bytes.extend_from_slice(&buffer[..read]);
                if bytes.len() > MAX_RESPONSE_BYTES {
                    return Err("sidecar response exceeds 1 MiB".into());
                }
                deadline = Instant::now() + RESPONSE_IDLE_TIMEOUT;
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    return Err("timed out waiting for sidecar response".into());
                }
            }
            Err(error) => return Err(format!("could not read sidecar response: {error}")),
        }
    }
}

fn generate_token() -> String {
    use sha2::{Digest, Sha256};
    let mut seed = [0u8; 32];
    let mut filled = false;
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        filled = file.read_exact(&mut seed).is_ok();
    }
    let mut hasher = Sha256::new();
    hasher.update(b"UOR-TAURI-SIDECAR-TOKEN-V1");
    hasher.update(seed);
    hasher.update(std::process::id().to_le_bytes());
    if let Ok(duration) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        hasher.update(duration.as_nanos().to_le_bytes());
    }
    if !filled {
        hasher.update(b"fallback-entropy");
    }
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod cancellation_registration_tests {
    use super::*;

    #[test]
    fn immediate_cancel_after_registration_is_not_overwritten() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let sidecar = SidecarProcess::start(
            &root.join("binaries/uor-chat-aarch64-apple-darwin"),
            &root.join("runtime/studio/pkg/synthetic-stack.lut"),
            &root.join("runtime/studio/pkg/tokenizer.json"),
        )
        .expect("start pinned uor-chat");
        let entered = Arc::new(Barrier::new(2));
        let release = Arc::new(Barrier::new(2));
        *REGISTRATION_PROBE
            .lock()
            .expect("install registration probe") = Some((entered.clone(), release.clone()));
        let worker = sidecar.clone();
        let pending = std::thread::spawn(move || {
            let mut events = Vec::new();
            let result = worker.stream_chat(
                "registration-race",
                r#"{"model":"uor-chat","messages":[{"role":"user","content":"Hi"}],"stream":false,"max_tokens":1}"#,
                |event| events.push(event),
            );
            (result, events)
        });
        entered.wait();
        sidecar
            .cancel("registration-race")
            .expect("cancel at registration boundary");
        release.wait();
        let (result, events) = pending.join().expect("registration thread joined");
        *REGISTRATION_PROBE
            .lock()
            .expect("remove registration probe") = None;
        let cancelled = match result {
            Ok(()) => events.iter().any(|event| {
                event
                    .error
                    .as_deref()
                    .is_some_and(|message| message.contains("cancelled"))
            }),
            Err(error) => error.contains("cancelled"),
        };
        assert!(cancelled, "registration overwrote the cancellation flag");
    }
}
