//! Loopback workbench for a validated native geometric artifact. HTTP, text
//! encoding and checkpoint I/O are host work outside the integer/table kernel.

use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{hash_map::RandomState, BTreeMap};
use std::hash::BuildHasher;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uor_r4_core::native_geometric::{Control, Model, Session, BOS, EOS};

const MAX_HEADERS: usize = 16 * 1024;
const MAX_BODY: usize = 2 * 1024 * 1024;
const MAX_PROMPT: usize = 256 * 1024;
const MAX_SESSIONS: usize = 32;
const MAX_SESSION_STORAGE: usize = 256 * 1024 * 1024;
const MAX_WORKERS: usize = 16;
const MAX_GENERATIONS: usize = 8;
const MAX_SAVED_FILES: usize = 256;
const MAX_SAVED_BYTES: u64 = 32 * 1024 * 1024;
const MAX_ARTIFACT: usize = 512 * 1024 * 1024;
const MAX_CHECKPOINT: usize = 1024 * 1024;
const MAX_OUTPUT: usize = 1024 * 1024;
const BACKEND: &str = "native-geometric-language-v1";
const BUSY: u8 = 1;
const CANCELLED: u8 = 2;
const CLOSED: u8 = 4;
type Reply = Result<Value, (u16, String)>;

#[derive(Debug)]
enum PersistenceError {
    CheckpointTooLarge { bytes: usize, limit: usize },
    Host(String),
}
impl std::fmt::Display for PersistenceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CheckpointTooLarge { bytes, limit } => write!(
                formatter,
                "checkpoint has {bytes} bytes, exceeding this service's {limit} byte restore limit"
            ),
            Self::Host(message) => formatter.write_str(message),
        }
    }
}
impl From<String> for PersistenceError {
    fn from(message: String) -> Self {
        Self::Host(message)
    }
}
impl From<&str> for PersistenceError {
    fn from(message: &str) -> Self {
        Self::Host(message.into())
    }
}
fn validate_checkpoint_size(bytes: &[u8]) -> Result<(), PersistenceError> {
    if bytes.len() > MAX_CHECKPOINT {
        return Err(PersistenceError::CheckpointTooLarge {
            bytes: bytes.len(),
            limit: MAX_CHECKPOINT,
        });
    }
    Ok(())
}

struct Slot {
    session: Mutex<Session>,
    activity: AtomicU8,
    // Fields drop in declaration order: retain the reservation until the
    // session buffers have been released, including after close removes its ID.
    _reservation: SessionReservation,
}
impl Slot {
    fn new(session: Session, reservation: SessionReservation) -> Self {
        Self {
            session: Mutex::new(session),
            activity: AtomicU8::new(0),
            _reservation: reservation,
        }
    }
    fn claim(&self) -> Result<BusyGuard<'_>, (u16, String)> {
        self.activity
            .compare_exchange(0, BUSY, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| (409, "session is already generating or being saved".into()))?;
        Ok(BusyGuard(&self.activity))
    }
    fn busy(&self) -> bool {
        self.activity.load(Ordering::Acquire) & BUSY != 0
    }
    fn cancelled(&self) -> bool {
        self.activity.load(Ordering::Acquire) & CANCELLED != 0
    }
}
struct SessionReservation(Arc<AtomicUsize>);
impl Drop for SessionReservation {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}
struct BusyGuard<'a>(&'a AtomicU8);
impl Drop for BusyGuard<'_> {
    fn drop(&mut self) {
        self.0.fetch_and(CLOSED, Ordering::Release);
    }
}

struct GenerationGuard<'a>(&'a AtomicUsize);
impl Drop for GenerationGuard<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

struct Service {
    model: Model,
    token_bytes: Vec<usize>,
    sessions: Mutex<BTreeMap<String, Arc<Slot>>>,
    session_directory: Option<PathBuf>,
    ids: AtomicU64,
    random: RandomState,
    persistence_lock: Mutex<()>,
    generations: AtomicUsize,
    session_limit: usize,
    session_storage_bytes: usize,
    session_reservations: Arc<AtomicUsize>,
}
impl Service {
    fn new(model: Model, session_directory: Option<&Path>) -> Result<Self, String> {
        let session_storage_bytes = model.session_storage_bytes().saturating_add(4096);
        let session_limit = MAX_SESSIONS.min(MAX_SESSION_STORAGE / session_storage_bytes);
        if session_limit == 0 {
            return Err(
                "artifact session storage exceeds this service's 256 MiB session budget".into(),
            );
        }
        // Decode each vocabulary entry once at host startup so a long learned
        // lexical piece cannot turn a bounded token request into unbounded text.
        let token_bytes = (0..model.vocabulary_size())
            .map(|token| {
                model
                    .decode(&[token as u32])
                    .map(|bytes| bytes.len())
                    .map_err(|e| e.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(directory) = session_directory {
            std::fs::create_dir_all(directory)
                .map_err(|e| format!("create session directory: {e}"))?;
        }
        Ok(Self {
            model,
            token_bytes,
            sessions: Mutex::new(BTreeMap::new()),
            session_directory: session_directory.map(Path::to_path_buf),
            ids: AtomicU64::new(0),
            random: RandomState::new(),
            persistence_lock: Mutex::new(()),
            generations: AtomicUsize::new(0),
            session_limit,
            session_storage_bytes,
            session_reservations: Arc::new(AtomicUsize::new(0)),
        })
    }
    // RandomState supplies independently randomized process-local keys.
    // Session IDs never accept user-controlled filesystem paths.
    fn fresh_id(&self) -> String {
        let serial = self.ids.fetch_add(1, Ordering::Relaxed);
        format!(
            "{:016x}{:016x}",
            self.random.hash_one((serial, 0_u8)),
            self.random.hash_one((serial, 1_u8))
        )
    }
    fn slot(&self, id: &str) -> Result<Arc<Slot>, (u16, String)> {
        validate_id(id)?;
        self.sessions
            .lock()
            .map_err(internal)?
            .get(id)
            .cloned()
            .ok_or_else(|| (404, "session is not loaded; create or restore it".into()))
    }
    fn insert(&self, create: impl FnOnce() -> Result<Session, (u16, String)>) -> Reply {
        let id = self.fresh_id();
        let mut slots = self.sessions.lock().map_err(internal)?;
        let reservation = self.reserve_session()?;
        // Serialize admission before allocating a possibly large memory index.
        let session = create()?;
        let state = session.state();
        self.persist(&id, &session).map_err(internal)?;
        slots.insert(id.clone(), Arc::new(Slot::new(session, reservation)));
        Ok(
            json!({"session_id":id,"artifact_cid":self.model.artifact_cid(),"state":state,"persisted":self.session_directory.is_some()}),
        )
    }
    fn reserve_session(&self) -> Result<SessionReservation, (u16, String)> {
        self.session_reservations
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |used| {
                (used < self.session_limit).then_some(used + 1)
            })
            .map_err(|_| (429, "session storage capacity reached; close an existing session and allow active requests to finish".into()))?;
        Ok(SessionReservation(Arc::clone(&self.session_reservations)))
    }
    fn persist(&self, id: &str, session: &Session) -> Result<(), PersistenceError> {
        if self.session_directory.is_none() {
            return Ok(());
        }
        let bytes = session.checkpoint().map_err(|e| e.to_string())?;
        self.persist_checkpoint(id, &bytes)
    }
    fn persist_checkpoint(&self, id: &str, bytes: &[u8]) -> Result<(), PersistenceError> {
        let Some(directory) = &self.session_directory else {
            return Ok(());
        };
        // The core /5 format permits larger snapshots. This host must not
        // replace a restorable checkpoint with one its restore path refuses.
        validate_checkpoint_size(bytes)?;
        let _persistence = self.persistence_lock.lock().map_err(|e| e.to_string())?;
        let temporary = directory.join(format!(".{}.tmp", self.fresh_id()));
        let destination = directory.join(format!("{id}.json"));
        let mut saved_files = 0_usize;
        let mut saved_bytes = 0_u64;
        let mut replaced_bytes = None;
        for entry in std::fs::read_dir(directory).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            saved_files = saved_files.saturating_add(1);
            if saved_files > MAX_SAVED_FILES {
                return Err("session directory has reached its saved-file limit".into());
            }
            let metadata = std::fs::symlink_metadata(entry.path()).map_err(|e| e.to_string())?;
            if !metadata.is_file() {
                return Err("session directory must contain only regular checkpoint files".into());
            }
            saved_bytes = saved_bytes.saturating_add(metadata.len());
            if entry.path() == destination {
                replaced_bytes = Some(metadata.len());
            }
        }
        if (replaced_bytes.is_none() && saved_files >= MAX_SAVED_FILES)
            || saved_bytes
                .saturating_sub(replaced_bytes.unwrap_or(0))
                .saturating_add(bytes.len() as u64)
                > MAX_SAVED_BYTES
        {
            return Err("saved-session capacity reached (256 files / 32 MiB); export or move old checkpoints before saving more".into());
        }
        let result = (|| {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            file.write_all(bytes)?;
            file.sync_all()?;
            std::fs::rename(&temporary, destination)
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        result.map_err(|e: std::io::Error| {
            PersistenceError::Host(format!("save native session: {e}"))
        })
    }
    fn route(&self, path: &str, body: &[u8]) -> Reply {
        match path {
            "/api/session" => {
                let request: NewSession = parse(body)?;
                self.insert(|| {
                    let mut session = self.model.session(request.control).map_err(internal)?;
                    session.observe(&self.model, BOS).map_err(internal)?;
                    Ok(session)
                })
            }
            "/api/generate" => self.generate(parse(body)?),
            "/api/cancel" => {
                let request: SessionId = parse(body)?;
                let slot = self.slot(&request.session_id)?;
                // No Session lock: cancellation stays visible during generation.
                let requested = slot
                    .activity
                    .fetch_update(Ordering::AcqRel, Ordering::Acquire, |activity| {
                        (activity & BUSY != 0).then_some(activity | CANCELLED)
                    })
                    .is_ok();
                Ok(json!({"cancellation_requested":requested,"busy":slot.busy()}))
            }
            "/api/export" => {
                let request: SessionId = parse(body)?;
                let slot = self.slot(&request.session_id)?;
                let _busy = slot.claim()?;
                let session = slot.session.lock().map_err(internal)?;
                let bytes = session.checkpoint().map_err(internal)?;
                let checkpoint: Value = serde_json::from_slice(&bytes).map_err(internal)?;
                Ok(json!({"artifact_cid":self.model.artifact_cid(),"checkpoint":checkpoint}))
            }
            "/api/import" => {
                let request: ImportSession = parse(body)?;
                let bytes = serde_json::to_vec(&request.checkpoint).map_err(internal)?;
                validate_checkpoint_size(&bytes).map_err(|e| (413, e.to_string()))?;
                self.insert(|| {
                    self.model
                        .restore_session(&bytes)
                        .map_err(|e| (400, e.to_string()))
                })
            }
            "/api/restore" => {
                let request: SessionId = parse(body)?;
                validate_id(&request.session_id)?;
                let mut slots = self.sessions.lock().map_err(internal)?;
                if let Some(slot) = slots.get(&request.session_id) {
                    let _busy = slot.claim()?;
                    let session = slot.session.lock().map_err(internal)?;
                    return Ok(
                        json!({"session_id":request.session_id,"state":session.state(),"artifact_cid":self.model.artifact_cid(),"persisted":self.session_directory.is_some()}),
                    );
                }
                let directory = self.session_directory.as_ref().ok_or_else(|| {
                    (
                        409,
                        "disk persistence is not configured; import a checkpoint instead".into(),
                    )
                })?;
                let reservation = self.reserve_session()?;
                let file = directory.join(format!("{}.json", request.session_id));
                let metadata = std::fs::symlink_metadata(&file)
                    .map_err(|_| (404, "saved session was not found".into()))?;
                if !metadata.is_file() {
                    return Err((400, "saved session must be a regular file".into()));
                }
                let bytes = read_bounded(&file, MAX_CHECKPOINT).map_err(internal)?;
                let session = self
                    .model
                    .restore_session(&bytes)
                    .map_err(|e| (400, e.to_string()))?;
                let state = session.state();
                slots.insert(
                    request.session_id.clone(),
                    Arc::new(Slot::new(session, reservation)),
                );
                Ok(
                    json!({"session_id":request.session_id,"state":state,"artifact_cid":self.model.artifact_cid(),"persisted":true}),
                )
            }
            "/api/close" => {
                let request: SessionId = parse(body)?;
                validate_id(&request.session_id)?;
                let mut slots = self.sessions.lock().map_err(internal)?;
                if let Some(slot) = slots.get(&request.session_id) {
                    slot.activity
                        .compare_exchange(0, CLOSED, Ordering::AcqRel, Ordering::Acquire)
                        .map_err(|_| {
                            (409, "cancel generation before closing the session".into())
                        })?;
                }
                slots.remove(&request.session_id);
                Ok(
                    json!({"closed":true,"saved_checkpoint_retained":self.session_directory.is_some()}),
                )
            }
            _ => Err((404, "unknown endpoint".into())),
        }
    }
    fn generate(&self, request: Generate) -> Reply {
        if request.prompt.len() > MAX_PROMPT || !(1..=4096).contains(&request.max_tokens) {
            return Err((
                400,
                "prompt limit is 256 KiB and max_tokens must be 1..=4096".into(),
            ));
        }
        self.generations
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                (n < MAX_GENERATIONS).then_some(n + 1)
            })
            .map_err(|_| {
                (
                    429,
                    "generation capacity reached; retry after an active generation finishes".into(),
                )
            })?;
        let _generation = GenerationGuard(&self.generations);
        let slot = self.slot(&request.session_id)?;
        let _busy = slot.claim()?;
        let prompt = self.model.encode(&request.prompt).map_err(internal)?;
        let mut session = slot.session.lock().map_err(internal)?;
        // Empty input continues a response stopped by the token/output limit.
        // New external input closes its selected-read state before observation.
        if !prompt.is_empty() {
            session.end_response(&self.model).map_err(internal)?;
        }
        let mut observed_prompt = 0;
        for &token in &prompt {
            if slot.cancelled() {
                break;
            }
            session.observe(&self.model, token).map_err(internal)?;
            observed_prompt += 1;
        }
        if !slot.cancelled()
            && (!prompt.is_empty()
                || session
                    .state()
                    .response
                    .is_none_or(|response| !response.active)
                    && session.state().values.is_none_or(|values| !values.active))
        {
            session.begin_response(&self.model).map_err(internal)?;
        }
        let mut output = Vec::new();
        let mut response_trace = Vec::new();
        let mut value_trace = Vec::new();
        let mut output_bytes = 0;
        let mut stop = "token_limit";
        for _ in 0..request.max_tokens {
            if slot.cancelled() {
                stop = "cancelled";
                break;
            }
            let prediction = session.predict(&self.model).map_err(internal)?;
            let next_bytes = self.token_bytes[prediction.token as usize];
            if next_bytes > MAX_OUTPUT - output_bytes {
                stop = "output_byte_limit";
                break;
            }
            if let Some(decision) = session.value_decision() {
                if value_trace.len() < 96 {
                    value_trace.push(decision);
                }
            }
            if let Some(decision) = session.response_decision() {
                if response_trace.len() < 96 {
                    response_trace.push(decision);
                }
            }
            if prediction.token == EOS {
                if session.response_decision().is_some()
                    || self.model.value_operator_version().is_some()
                {
                    session.observe(&self.model, EOS).map_err(internal)?;
                }
                stop = "end_of_sequence";
                break;
            }
            session
                .observe(&self.model, prediction.token)
                .map_err(internal)?;
            output.push(prediction.token);
            output_bytes += next_bytes;
        }
        let bytes = self.model.decode(&output).map_err(internal)?;
        let text = String::from_utf8_lossy(&bytes);
        let utf8_valid = std::str::from_utf8(&bytes).is_ok();
        let persist_error = self
            .persist(&request.session_id, &session)
            .err()
            .map(|error| error.to_string());
        Ok(
            json!({"backend":BACKEND,"artifact_cid":self.model.artifact_cid(),"session_id":request.session_id,
            "memory_read_version":self.model.memory_read_version(),
            "text":text,"utf8_valid":utf8_valid,"bytes":bytes,"tokens":output,"stop":stop,
            "response_trace":response_trace,"value_trace":value_trace,
            "prompt_tokens":prompt.len(),"observed_prompt_tokens":observed_prompt,
            "state":session.state(),"session_work":session.work,
            "persisted":self.session_directory.is_some() && persist_error.is_none(),"persistence_error":persist_error}),
        )
    }
    fn info(&self) -> Value {
        json!({"backend":BACKEND,"artifact_cid":self.model.artifact_cid(),
            "uor_model_address":self.model.uor_model_address(),"readout_version":self.model.readout_version(),
            "memory_read_version":self.model.memory_read_version(),
            "status":"experimental_native_geometric_language_model",
            "config":self.model.config(),"training":self.model.training(),"vocabulary_size":self.model.vocabulary_size(),
            "geometry_identities":self.model.geometry_identities(),
            "anchor_identities":self.model.anchor_identities(),
            "kernel":"integer/table lookup; host HTTP, text and checkpoint handling allocate",
            "session_limit":self.session_limit,"session_storage_bytes":self.session_storage_bytes,
            "session_storage_budget_bytes":MAX_SESSION_STORAGE,
            "live_session_reservations":self.session_reservations.load(Ordering::Acquire),
            "generation_limit":4096,"prompt_byte_limit":MAX_PROMPT,"output_byte_limit":MAX_OUTPUT,
            "concurrent_generation_limit":MAX_GENERATIONS,"saved_file_limit":MAX_SAVED_FILES,"saved_byte_limit":MAX_SAVED_BYTES,
            "checkpoint_byte_limit":MAX_CHECKPOINT,
            "disk_persistence":self.session_directory.is_some()})
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NewSession {
    #[serde(default)]
    control: Control,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionId {
    session_id: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Generate {
    session_id: String,
    prompt: String,
    max_tokens: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ImportSession {
    checkpoint: Value,
}
fn parse<T: serde::de::DeserializeOwned>(body: &[u8]) -> Result<T, (u16, String)> {
    serde_json::from_slice(body).map_err(|e| (400, format!("invalid request: {e}")))
}
fn internal(error: impl std::fmt::Display) -> (u16, String) {
    (500, error.to_string())
}
fn validate_id(id: &str) -> Result<(), (u16, String)> {
    if id.len() == 32 && id.bytes().all(|b| b.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err((400, "invalid session identifier".into()))
    }
}
fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
    if file.metadata().map_err(|e| e.to_string())?.len() > limit as u64 {
        return Err(format!("file exceeds {limit} byte limit"));
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > limit {
        return Err(format!("file exceeds {limit} byte limit"));
    }
    Ok(bytes)
}

struct Request {
    method: String,
    path: String,
    body: Vec<u8>,
}
fn read_request(stream: &mut TcpStream, address: SocketAddr) -> Result<Request, (u16, String)> {
    let deadline = Instant::now() + Duration::from_secs(10);
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(internal)?;
    let mut header = Vec::new();
    while !header.ends_with(b"\r\n\r\n") {
        if header.len() >= MAX_HEADERS {
            return Err((431, "headers exceed 16 KiB".into()));
        }
        let mut byte = [0_u8; 1];
        if read_before_deadline(stream, &mut byte, deadline)? != 1 {
            return Err((400, "incomplete HTTP headers".into()));
        }
        header.push(byte[0]);
    }
    let header = std::str::from_utf8(&header)
        .map_err(|_| (400, "headers must be UTF-8 ASCII-compatible text".into()))?;
    let mut lines = header.split("\r\n");
    let parts: Vec<_> = lines.next().unwrap_or("").split_whitespace().collect();
    if parts.len() != 3 || parts[2] != "HTTP/1.1" || !parts[1].starts_with('/') {
        return Err((400, "expected an HTTP/1.1 origin-form request".into()));
    }
    if parts[0] != "GET" && parts[0] != "POST" {
        return Err((405, "only GET and POST are supported".into()));
    }
    let mut headers = BTreeMap::new();
    for line in lines.filter(|line| !line.is_empty()) {
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| (400, "malformed header".into()))?;
        let key = key.to_ascii_lowercase();
        if headers.insert(key, value.trim()).is_some() {
            return Err((400, "duplicate headers are not supported".into()));
        }
    }
    let host = headers
        .get("host")
        .copied()
        .ok_or_else(|| (400, "Host header is required".into()))?;
    if host != format!("127.0.0.1:{}", address.port())
        && host != format!("localhost:{}", address.port())
    {
        return Err((403, "Host must name this loopback listener".into()));
    }
    if let Some(origin) = headers.get("origin") {
        if *origin != format!("http://{host}") {
            return Err((403, "Origin must match this loopback listener".into()));
        }
    }
    if headers.contains_key("transfer-encoding") {
        return Err((400, "Transfer-Encoding is not supported".into()));
    }
    let length = headers
        .get("content-length")
        .map(|s| s.parse::<usize>())
        .transpose()
        .map_err(|_| (400, "invalid Content-Length".into()))?
        .unwrap_or(0);
    if length > MAX_BODY {
        return Err((413, "request body exceeds 2 MiB".into()));
    }
    if parts[0] == "POST" {
        if !headers.contains_key("content-length") {
            return Err((411, "Content-Length is required".into()));
        }
        let content_type = headers
            .get("content-type")
            .copied()
            .unwrap_or("")
            .split(';')
            .next()
            .unwrap_or("")
            .trim();
        if !content_type.eq_ignore_ascii_case("application/json") {
            return Err((415, "POST requires application/json".into()));
        }
    } else if length != 0 {
        return Err((400, "GET request body is not supported".into()));
    }
    let mut body = vec![0; length];
    let mut consumed = 0;
    while consumed < length {
        let count = read_before_deadline(stream, &mut body[consumed..], deadline)?;
        if count == 0 {
            return Err((400, "incomplete HTTP body".into()));
        }
        consumed += count;
    }
    Ok(Request {
        method: parts[0].into(),
        path: parts[1].into(),
        body,
    })
}
fn read_before_deadline(
    stream: &mut TcpStream,
    buffer: &mut [u8],
    deadline: Instant,
) -> Result<usize, (u16, String)> {
    let remaining = deadline
        .checked_duration_since(Instant::now())
        .ok_or_else(|| (400, "HTTP request timed out".into()))?;
    if remaining.is_zero() {
        return Err((400, "HTTP request timed out".into()));
    }
    stream.set_read_timeout(Some(remaining)).map_err(internal)?;
    stream
        .read(buffer)
        .map_err(|e| (400, format!("incomplete HTTP request: {e}")))
}
fn respond(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        411 => "Length Required",
        413 => "Content Too Large",
        415 => "Unsupported Media Type",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        _ => "Internal Server Error",
    };
    write!(stream,"HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nContent-Security-Policy: default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'\r\n\r\n",body.len())?;
    stream.write_all(body)
}
fn handle(mut stream: TcpStream, address: SocketAddr, service: &Service) {
    let reply = match read_request(&mut stream, address) {
        Ok(request) if request.method == "GET" && request.path == "/" => {
            let _ = respond(
                &mut stream,
                200,
                "text/html; charset=utf-8",
                WORKBENCH.as_bytes(),
            );
            return;
        }
        Ok(request)
            if request.method == "GET"
                && (request.path == "/api/info" || request.path == "/health") =>
        {
            Ok(service.info())
        }
        Ok(request) if request.method == "POST" => service.route(&request.path, &request.body),
        Ok(_) => Err((404, "unknown endpoint".into())),
        Err(error) => Err(error),
    };
    let (status, value) = match reply {
        Ok(value) => (200, value),
        Err((status, error)) => (status, json!({"error":error,"backend":BACKEND})),
    };
    if let Ok(bytes) = serde_json::to_vec(&value) {
        let _ = respond(
            &mut stream,
            status,
            "application/json; charset=utf-8",
            &bytes,
        );
        if status >= 400 {
            finish_rejection(&mut stream);
        }
    }
}

fn finish_rejection(stream: &mut TcpStream) {
    // Deliver the rejection before closing an unread request body. Immediate
    // close can reset TCP and discard the HTTP error on macOS. The grace
    // drain has both a byte cap and total deadline.
    let _ = stream.shutdown(Shutdown::Write);
    let deadline = Instant::now() + Duration::from_millis(100);
    let mut remaining = MAX_BODY + MAX_HEADERS;
    let mut buffer = [0_u8; 4096];
    while remaining > 0 {
        let limit = remaining.min(buffer.len());
        match read_before_deadline(stream, &mut buffer[..limit], deadline) {
            Ok(0) | Err(_) => break,
            Ok(count) => remaining -= count,
        }
    }
}

/// Loads and validates the exact artifact before binding the loopback listener.
pub fn serve(
    artifact_path: &Path,
    port: u16,
    session_directory: Option<&Path>,
) -> Result<(), String> {
    let bytes = read_bounded(artifact_path, MAX_ARTIFACT)?;
    let model = Model::from_bytes(&bytes).map_err(|e| e.to_string())?;
    let service = Arc::new(Service::new(model, session_directory)?);
    let listener =
        TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).map_err(|e| e.to_string())?;
    let address = listener.local_addr().map_err(|e| e.to_string())?;
    println!("Native geometric workbench: http://{address}");
    println!(
        "Artifact: {} (experimental language model)",
        service.model.artifact_cid()
    );
    std::io::stdout().flush().map_err(|e| e.to_string())?;
    let workers = Arc::new(AtomicUsize::new(0));
    for stream in listener.incoming() {
        let mut stream = stream.map_err(|e| e.to_string())?;
        if workers
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                (n < MAX_WORKERS).then_some(n + 1)
            })
            .is_err()
        {
            let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
            let _ = respond(
                &mut stream,
                429,
                "application/json",
                b"{\"error\":\"server is busy\"}",
            );
            finish_rejection(&mut stream);
            continue;
        }
        let service = Arc::clone(&service);
        let count = Arc::clone(&workers);
        if let Err(error) = std::thread::Builder::new()
            .name("native-geometric-http".into())
            .spawn(move || {
                struct Worker(Arc<AtomicUsize>);
                impl Drop for Worker {
                    fn drop(&mut self) {
                        self.0.fetch_sub(1, Ordering::AcqRel);
                    }
                }
                let _worker = Worker(count);
                handle(stream, address, &service);
            })
        {
            workers.fetch_sub(1, Ordering::AcqRel);
            return Err(format!("start HTTP worker: {error}"));
        }
    }
    Ok(())
}

const WORKBENCH: &str = r#"<!doctype html>
<html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>R4 · Native geometric workbench</title>
<style>body{margin:0;background:#0d1520;color:#e4edf6;font:16px system-ui,sans-serif}main{max-width:960px;margin:48px auto;padding:0 24px}h1{font-size:30px;letter-spacing:-.6px}small,.muted{color:#9fafc1}label{display:block;margin:24px 0 8px}textarea,pre{box-sizing:border-box;width:100%;border:1px solid #3a4d60;border-radius:8px;background:#142130;color:#eef4fa;padding:16px;font:14px/1.6 ui-monospace,monospace}textarea{min-height:200px;resize:vertical}pre{min-height:100px;white-space:pre-wrap;overflow-wrap:anywhere}button,input{background:#20364b;color:#e4edf6;border:1px solid #50677e;border-radius:6px;padding:9px 12px}button{cursor:pointer;margin:8px 6px 8px 0}button:disabled{opacity:.4;cursor:default}#generate{background:#79c7b5;color:#102421;border-color:#79c7b5}input[type=number]{width:75px}#status{min-height:24px;color:#acdaca}details{margin:20px 0}footer{margin:40px 0;color:#9fafc1;font-size:13px}</style>
<main><small>UOR–R4 / LOCAL RESEARCH WORKBENCH</small><h1>Native geometric language</h1>
<p class="muted">Write a prompt or code fragment. The loaded model continues the text using its learned geometric tables. This is an experimental language model; coding and reasoning quality remain under evaluation.</p>
<p id="model" class="muted">Loading artifact information…</p>
<label for="prompt">Text to append to this session</label><textarea id="prompt" spellcheck="false" placeholder="Enter text or code to continue…"></textarea>
<div><button id="generate">Generate continuation</button><button id="cancel" disabled>Stop</button><label style="display:inline" for="limit">Maximum new tokens </label><input id="limit" type="number" min="1" max="4096" value="128"></div>
<p id="status" role="status"></p><label>Generated continuation</label><pre id="output"></pre>
<button id="reset">New session</button><button id="save">Export session</button><button id="load">Import session</button><input id="file" type="file" accept="application/json,.json" hidden>
<details><summary>Geometry, work and artifact details</summary><pre id="details"></pre></details>
<footer>Runs on 127.0.0.1 with the exact native artifact shown above. Generated code is displayed as text. Exported checkpoints can resume only with the same artifact.</footer></main>
<script>
const el=id=>document.getElementById(id);let session=null,running=false,info=null;
async function api(path,body){const r=await fetch(path,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(body)});const v=await r.json();if(!r.ok)throw Error(v.error||r.statusText);return v}
function status(text){el('status').textContent=text}
function busy(on,cancellable=false){running=on;for(const id of ['generate','reset','save','load'])el(id).disabled=on;el('cancel').disabled=!on||!cancellable}
function adopt(value){session=value.session_id;sessionStorage.setItem('r4-native-session',session);el('details').textContent=JSON.stringify(value,null,2)}
async function fresh(){busy(true);try{if(session)await api('/api/close',{session_id:session});adopt(await api('/api/session',{}));el('output').textContent='';status('New session ready.')}finally{busy(false)}}
el('generate').onclick=async()=>{busy(true,true);status('Generating…');try{const v=await api('/api/generate',{session_id:session,prompt:el('prompt').value,max_tokens:Number(el('limit').value)});el('output').textContent=v.text;el('details').textContent=JSON.stringify(v,null,2);status(`${v.tokens.length} new tokens · ${v.stop}${v.persistence_error?' · save failed: '+v.persistence_error:''}${v.utf8_valid?'':' · output includes invalid UTF-8; exact bytes are in details'}`);if(v.observed_prompt_tokens===v.prompt_tokens)el('prompt').value='';else status('Stopped while reading your prompt. Start a new session before sending the full prompt again.')}catch(e){status(e.message)}finally{busy(false)}};
el('cancel').onclick=async()=>{try{await api('/api/cancel',{session_id:session});status('Stop requested; returning completed tokens…')}catch(e){status(e.message)}};
el('reset').onclick=()=>fresh().catch(e=>status(e.message));
el('save').onclick=async()=>{busy(true);try{const v=await api('/api/export',{session_id:session});const u=URL.createObjectURL(new Blob([JSON.stringify(v,null,2)],{type:'application/json'}));const a=document.createElement('a');a.href=u;a.download='r4-native-session.json';a.click();URL.revokeObjectURL(u);status('Session exported.')}catch(e){status(e.message)}finally{busy(false)}};
el('load').onclick=()=>el('file').click();el('file').onchange=async()=>{busy(true);try{const f=el('file').files[0];if(!f)return;if(f.size>2097152)throw Error('Checkpoint exceeds 2 MiB');const v=JSON.parse(await f.text());const imported=await api('/api/import',{checkpoint:v.checkpoint||v});if(session)await api('/api/close',{session_id:session});adopt(imported);el('output').textContent='';status('Session imported.')}catch(e){status(e.message)}finally{el('file').value='';busy(false)}};
(async()=>{busy(true);try{const r=await fetch('/api/info');info=await r.json();if(!r.ok)throw Error(info.error);el('model').textContent=`${info.backend} · ${info.config.context_tokens} context tokens · artifact ${info.artifact_cid}`;const saved=sessionStorage.getItem('r4-native-session');if(saved){try{adopt(await api('/api/restore',{session_id:saved}));status('Session restored.');return}catch{sessionStorage.removeItem('r4-native-session')}}await fresh()}catch(e){status(e.message)}finally{busy(false)}})();
</script></html>"#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};
    use std::sync::OnceLock;
    use uor_r4_core::native_geometric::{Config, Document, Trainer};

    fn model() -> Model {
        static MODEL: OnceLock<Model> = OnceLock::new();
        MODEL.get_or_init(|| {
            let docs = [Document { id:"service-fixture".into(),
                text:"red fox keeps amber stone. blue bird keeps green seed. red fox finds amber stone.".into() }];
            let mut trainer = Trainer::new(Config { context_tokens:8,candidate_limit:4,
                max_lexical_pieces:64,postings_per_row:4,..Config::default() },&docs).unwrap();
            trainer.train_documents(&docs).unwrap();
            let fitted = trainer.compile().unwrap();
            Model::from_bytes(&fitted.to_bytes().unwrap()).unwrap()
        }).clone()
    }

    fn service() -> Arc<Service> {
        Arc::new(Service::new(model(), None).unwrap())
    }

    fn response_model() -> Model {
        use uor_r4_core::native_geometric::{
            MemoryReadFitConfig, MemoryReadSchedule, MemoryReadSupervision, MemoryReadTokenSpan,
            MemoryReadTrainer,
        };
        let construction = [Document {
            id: "response-service-count".into(),
            text: "red fox red fox. blue bird blue bird.".into(),
        }];
        let mut count = Trainer::new(
            Config {
                context_tokens: 32,
                candidate_limit: 8,
                max_lexical_pieces: 64,
                ..Config::default()
            },
            &construction,
        )
        .unwrap();
        count.train_documents(&construction).unwrap();
        let baseline = count.compile().unwrap();
        let documents = [Document {
            id: "response-service-fit".into(),
            text: "red fox red fox red fox. blue bird blue bird blue bird.".into(),
        }];
        let end = baseline.encode(&documents[0].text).unwrap().len() + 1;
        let supervision = MemoryReadSupervision::new(
            &baseline,
            &documents,
            vec![vec![MemoryReadTokenSpan { start: 2, end }]],
        )
        .unwrap();
        let mut trainer = MemoryReadTrainer::new_with_response_state(
            &baseline,
            &documents,
            MemoryReadFitConfig {
                advance_response_path: false,
                query_tokens: 4,
                source_offsets: 2,
                postings_per_address: 2,
                candidate_limit: 16,
                max_positions: 32,
                epochs: 1,
                max_features: 4096,
            },
            MemoryReadSchedule {
                total_positions: 32,
                batch_positions: 32,
            },
            true,
            supervision,
        )
        .unwrap();
        for _ in 0..1000 {
            if trainer.is_complete() {
                break;
            }
            trainer.advance(8, Duration::from_secs(10)).unwrap();
        }
        assert!(trainer.is_complete());
        trainer.finish().unwrap().0
    }

    fn exchange(service: Arc<Service>, request: impl FnOnce(u16) -> Vec<u8>) -> String {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        assert_ne!(address.port(), 0);
        let worker = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle(stream, address, &service);
        });
        let mut stream = TcpStream::connect(address).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        stream.write_all(&request(address.port())).unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
        // Read the declared HTTP response, not until TCP EOF: a server that
        // rejects an unread request body may reset the connection afterwards.
        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        loop {
            let mut line = String::new();
            assert_ne!(reader.read_line(&mut line).unwrap(), 0);
            response.push_str(&line);
            if line == "\r\n" {
                break;
            }
        }
        let length = response
            .lines()
            .find_map(|line| line.strip_prefix("Content-Length: "))
            .unwrap()
            .trim()
            .parse::<usize>()
            .unwrap();
        let mut body = vec![0; length];
        reader.read_exact(&mut body).unwrap();
        response.push_str(std::str::from_utf8(&body).unwrap());
        worker.join().unwrap();
        response
    }

    fn post(service: Arc<Service>, path: &str, body: Value) -> Value {
        let bytes = serde_json::to_vec(&body).unwrap();
        let response = exchange(service, |port| {
            let mut request = format!("POST {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nOrigin: http://127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",bytes.len()).into_bytes();
            request.extend(bytes);
            request
        });
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        serde_json::from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap()
    }

    fn id(value: &Value) -> String {
        value["session_id"].as_str().unwrap().into()
    }

    #[test]
    fn http_generation_uses_loaded_artifact_and_isolates_sessions() {
        let service = service();
        let health = exchange(Arc::clone(&service), |port| {
            format!("GET /health HTTP/1.1\r\nHost: localhost:{port}\r\n\r\n").into_bytes()
        });
        assert!(health.contains(service.model.artifact_cid()));
        assert!(health.contains(BACKEND));
        let page = exchange(Arc::clone(&service), |port| {
            format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n").into_bytes()
        });
        assert!(page.contains("<textarea"));
        assert!(page.contains("frame-ancestors 'none'"));
        let first = id(&post(Arc::clone(&service), "/api/session", json!({})));
        let second = id(&post(Arc::clone(&service), "/api/session", json!({})));
        assert_ne!(first, second);
        let generated = post(
            Arc::clone(&service),
            "/api/generate",
            json!({"session_id":first,"prompt":"red fox","max_tokens":4}),
        );
        let mut expected = service.model.session(Control::Full).unwrap();
        expected.observe(&service.model, BOS).unwrap();
        for token in service.model.encode("red fox").unwrap() {
            expected.observe(&service.model, token).unwrap();
        }
        let mut tokens = Vec::new();
        for _ in 0..4 {
            let prediction = expected.predict(&service.model).unwrap();
            if prediction.token == EOS {
                break;
            }
            expected.observe(&service.model, prediction.token).unwrap();
            tokens.push(prediction.token);
        }
        assert_eq!(generated["tokens"], json!(tokens));
        assert_eq!(
            generated["bytes"],
            json!(service.model.decode(&tokens).unwrap())
        );
        assert_eq!(generated["state"], json!(expected.state()));
        assert_eq!(generated["session_work"], json!(expected.work));
        assert_eq!(generated["artifact_cid"], service.model.artifact_cid());
        assert_eq!(
            service
                .slot(&second)
                .unwrap()
                .session
                .lock()
                .unwrap()
                .state()
                .tokens_seen,
            1
        );
        assert!(
            service
                .slot(&first)
                .unwrap()
                .session
                .lock()
                .unwrap()
                .state()
                .tokens_seen
                > 1
        );
    }

    #[test]
    fn http_response_query_survives_token_limit_and_restarts_for_external_input() {
        let model = response_model();
        let prompt = ["red", "red fox", "blue", "blue bird"]
            .into_iter()
            .find(|prompt| model.generate(prompt, 1, Control::Full).unwrap().stop == "token_budget")
            .expect("fixture must exercise a non-EOS first step");
        let expected = model.generate(prompt, 2, Control::Full).unwrap();
        let service = Arc::new(Service::new(model, None).unwrap());
        let session_id = id(&post(Arc::clone(&service), "/api/session", json!({})));
        let isolated_id = id(&post(Arc::clone(&service), "/api/session", json!({})));
        let first = post(
            Arc::clone(&service),
            "/api/generate",
            json!({"session_id":session_id,"prompt":prompt,"max_tokens":1}),
        );
        assert_eq!(first["stop"], "token_limit");
        assert_eq!(first["session_work"]["response_query_captures"], 1);
        assert_eq!(first["state"]["response"]["active"], true);
        let second = post(
            Arc::clone(&service),
            "/api/generate",
            json!({"session_id":session_id,"prompt":"","max_tokens":1}),
        );
        let mut tokens = first["tokens"].as_array().unwrap().clone();
        tokens.extend(second["tokens"].as_array().unwrap().iter().cloned());
        assert_eq!(json!(tokens), json!(expected.token_ids));
        assert_eq!(second["state"], json!(expected.state));
        assert_eq!(second["session_work"], json!(expected.work));
        assert_eq!(second["session_work"]["response_query_captures"], 1);
        let restarted = post(
            Arc::clone(&service),
            "/api/generate",
            json!({"session_id":session_id,"prompt":"red fox","max_tokens":1}),
        );
        assert_eq!(restarted["session_work"]["response_query_captures"], 2);
        let isolated = service.slot(&isolated_id).unwrap();
        let isolated = isolated.session.lock().unwrap();
        assert_eq!(isolated.work.response_query_captures, 0);
        assert!(!isolated.state().response.unwrap().active);
    }

    #[test]
    fn http_typed_numeral_continues_across_requests_and_imported_snapshot() {
        typed_numeral_continuation(false);
        typed_numeral_continuation(true);
    }

    fn typed_numeral_continuation(lexeme_cues: bool) {
        use uor_r4_core::native_geometric::{ValueExample, ValueFitConfig};
        // A tiny construction fit supplies the public values-enabled artifact.
        // This test concerns HTTP/state continuity, not held-out arithmetic.
        let examples = (0..4)
            .map(|index| ValueExample {
                id: format!("value-service-fit-{index}"),
                prompt: format!("left = {}; right = 4; total:", 13 + index),
                response: (17 + index).to_string(),
            })
            .collect::<Vec<_>>();
        let baseline = model();
        let config = ValueFitConfig {
            epochs: 32,
            learning_rate: 0.25,
            max_features: 4096,
        };
        let (fitted, report) = if lexeme_cues {
            baseline.fit_values_with_lexeme_cues(&examples, config)
        } else {
            baseline.fit_values(&examples, config)
        }
        .unwrap();
        assert_eq!(report.reachable_numeric_targets, 4);
        assert!(report.continuation_positions > 0);
        let model = Model::from_bytes(&fitted.to_bytes().unwrap()).unwrap();
        let service = Arc::new(Service::new(model, None).unwrap());
        let complete_id = id(&post(Arc::clone(&service), "/api/session", json!({})));
        let split_id = id(&post(Arc::clone(&service), "/api/session", json!({})));
        let prompt = &examples[0].prompt;
        let complete = post(
            Arc::clone(&service),
            "/api/generate",
            json!({"session_id":complete_id,"prompt":prompt,"max_tokens":2}),
        );
        assert_eq!(complete["tokens"], json!([51, 57]));
        assert_eq!(complete["bytes"], json!([b'1', b'7']));
        assert_eq!(complete["text"], "17");
        let first = post(
            Arc::clone(&service),
            "/api/generate",
            json!({"session_id":split_id,"prompt":prompt,"max_tokens":1}),
        );
        assert_eq!(first["stop"], "token_limit");
        assert_eq!(first["tokens"], json!([51]));
        assert_eq!(first["state"]["values"]["active"], true);
        assert_eq!(first["state"]["values"]["emission_cursor"], 1);
        let write = first["state"]["values"]["committed_write"].clone();
        let exported = post(
            Arc::clone(&service),
            "/api/export",
            json!({"session_id":split_id}),
        );
        assert_eq!(
            exported["checkpoint"]["schema"],
            "uor-r4.native-geometric-session/3"
        );
        let restored_id = id(&post(
            Arc::clone(&service),
            "/api/import",
            json!({"checkpoint":exported["checkpoint"]}),
        ));
        for session_id in [split_id, restored_id] {
            let second = post(
                Arc::clone(&service),
                "/api/generate",
                json!({"session_id":session_id,"prompt":"","max_tokens":1}),
            );
            assert_eq!(second["observed_prompt_tokens"], 0);
            assert_eq!(second["tokens"], json!([57]));
            assert_eq!(second["value_trace"][0]["write_id"], write);
            assert_eq!(second["value_trace"][0]["cursor"], 1);
            let mut tokens = first["tokens"].as_array().unwrap().clone();
            tokens.extend(second["tokens"].as_array().unwrap().iter().cloned());
            let mut trace = first["value_trace"].as_array().unwrap().clone();
            trace.extend(second["value_trace"].as_array().unwrap().iter().cloned());
            assert_eq!(json!(tokens), complete["tokens"]);
            assert_eq!(json!(trace), complete["value_trace"]);
            assert_eq!(second["state"], complete["state"]);
            assert_eq!(second["session_work"], complete["session_work"]);
            assert_eq!(second["session_work"]["values"]["derived_writes"], 1);
        }
    }

    #[test]
    fn http_rejects_foreign_origins_ambiguous_lengths_and_oversized_requests() {
        let service = service();
        for (suffix,expected) in [
            ("Origin: https://foreign.example\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}",403),
            ("Content-Type: text/plain\r\nContent-Length: 2\r\n\r\n{}",415),
            ("Content-Type: application/json\r\nContent-Length: 2\r\nContent-Length: 2\r\n\r\n{}",400),
            ("Content-Type: application/json\r\nContent-Length: 2097153\r\n\r\n",413),
            ("Content-Type: application/json\r\nTransfer-Encoding: chunked\r\n\r\n",400),
        ] {
            let response = exchange(Arc::clone(&service),|port|
                format!("POST /api/session HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n{suffix}").into_bytes());
            assert!(response.starts_with(&format!("HTTP/1.1 {expected}")),"{response}");
        }
        let response = exchange(service, |_| {
            b"GET /health HTTP/1.1\r\nHost: foreign.example\r\n\r\n".to_vec()
        });
        assert!(response.starts_with("HTTP/1.1 403"));
    }

    #[test]
    fn cancellation_does_not_wait_for_the_session_lock_or_leak_to_next_turn() {
        let service = service();
        let first = id(&post(Arc::clone(&service), "/api/session", json!({})));
        let slot = service.slot(&first).unwrap();
        let locked = slot.session.lock().unwrap();
        let generated_service = Arc::clone(&service);
        let generated_id = first.clone();
        let generation = std::thread::spawn(move || {
            post(
                generated_service,
                "/api/generate",
                json!({"session_id":generated_id,"prompt":"red fox","max_tokens":4096}),
            )
        });
        let deadline = Instant::now() + Duration::from_secs(5);
        while !slot.busy() && Instant::now() < deadline {
            std::thread::yield_now();
        }
        assert!(slot.busy());
        assert!(slot.claim().is_err());
        let cancelled = post(
            Arc::clone(&service),
            "/api/cancel",
            json!({"session_id":first}),
        );
        assert_eq!(cancelled["cancellation_requested"], true);
        assert!(slot.cancelled());
        drop(locked);
        let response = generation.join().unwrap();
        assert_eq!(response["stop"], "cancelled");
        assert_eq!(response["observed_prompt_tokens"], 0);
        assert_eq!(response["state"]["tokens_seen"], 1);
        assert!(!slot.busy());
        assert!(!slot.cancelled());
        let next = post(
            service,
            "/api/generate",
            json!({"session_id":first,"prompt":"red fox","max_tokens":1}),
        );
        assert_ne!(next["stop"], "cancelled");
        assert!(next["observed_prompt_tokens"].as_u64().unwrap() > 0);
    }

    #[test]
    fn checkpoints_roundtrip_disk_export_import_and_enforce_artifact_identity() {
        let directory = std::env::temp_dir().join(format!(
            "r4-native-service-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let service = Arc::new(Service::new(model(), Some(&directory)).unwrap());
        let first = id(&post(Arc::clone(&service), "/api/session", json!({})));
        post(
            Arc::clone(&service),
            "/api/generate",
            json!({"session_id":first,"prompt":"red fox keeps amber stone. blue bird keeps green seed.","max_tokens":3}),
        );
        let exported = post(
            Arc::clone(&service),
            "/api/export",
            json!({"session_id":first}),
        );
        let imported = id(&post(
            Arc::clone(&service),
            "/api/import",
            json!({"checkpoint":exported["checkpoint"]}),
        ));
        assert_ne!(first, imported);
        let expected = service
            .slot(&first)
            .unwrap()
            .session
            .lock()
            .unwrap()
            .state();
        assert_eq!(
            service
                .slot(&imported)
                .unwrap()
                .session
                .lock()
                .unwrap()
                .state(),
            expected
        );
        let old_slot = service.slot(&first).unwrap();
        post(
            Arc::clone(&service),
            "/api/close",
            json!({"session_id":first}),
        );
        assert!(
            old_slot.claim().is_err(),
            "closed slot cannot be used by an in-flight stale lookup"
        );
        let restored = post(
            Arc::clone(&service),
            "/api/restore",
            json!({"session_id":first}),
        );
        assert_eq!(restored["state"], json!(expected));
        let a = post(
            Arc::clone(&service),
            "/api/generate",
            json!({"session_id":first,"prompt":"","max_tokens":5}),
        );
        let b = post(
            Arc::clone(&service),
            "/api/generate",
            json!({"session_id":imported,"prompt":"","max_tokens":5}),
        );
        assert_eq!(a["tokens"], b["tokens"]);
        let mut bad = exported["checkpoint"].clone();
        bad["artifact_cid"] = json!("different-artifact");
        assert_eq!(
            service
                .route(
                    "/api/import",
                    &serde_json::to_vec(&json!({"checkpoint":bad})).unwrap()
                )
                .unwrap_err()
                .0,
            400
        );
        assert_eq!(
            service
                .route("/api/restore", br#"{"session_id":"../outside"}"#)
                .unwrap_err()
                .0,
            400
        );
        drop(service);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn closed_session_keeps_storage_reserved_until_pending_references_drop() {
        let mut service = Service::new(model(), None).unwrap();
        service.session_limit = 1;
        let first = id(&service.route("/api/session", b"{}").unwrap());
        let pending_request = service.slot(&first).unwrap();
        service
            .route(
                "/api/close",
                &serde_json::to_vec(&json!({"session_id":first})).unwrap(),
            )
            .unwrap();
        assert!(pending_request.claim().is_err());
        assert_eq!(service.session_reservations.load(Ordering::Acquire), 1);
        assert_eq!(service.route("/api/session", b"{}").unwrap_err().0, 429);
        drop(pending_request);
        assert_eq!(service.session_reservations.load(Ordering::Acquire), 0);
        assert!(service.route("/api/session", b"{}").is_ok());
    }

    #[test]
    fn startup_rejects_non_native_artifact_and_session_capacity_is_bounded() {
        let path = std::env::temp_dir().join(format!(
            "r4-native-invalid-artifact-{}.json",
            std::process::id()
        ));
        std::fs::write(&path, b"{}").unwrap();
        assert!(serve(&path, 0, None).is_err());
        std::fs::remove_file(path).unwrap();
        let service = service();
        for _ in 0..MAX_SESSIONS {
            service.route("/api/session", b"{}").unwrap();
        }
        assert_eq!(service.route("/api/session", b"{}").unwrap_err().0, 429);
        assert_eq!(service.route("/api/generate",br#"{"session_id":"00000000000000000000000000000000","prompt":"","max_tokens":0}"#).unwrap_err().0,400);
    }

    #[test]
    fn checkpoint_size_limit_preserves_saved_state_and_import_capacity() {
        let directory = std::env::temp_dir().join(format!(
            "r4-native-service-checkpoint-limit-{}",
            std::process::id()
        ));
        let service = Service::new(model(), Some(&directory)).unwrap();
        let created = service.route("/api/session", b"{}").unwrap();
        let first = id(&created);
        let saved = directory.join(format!("{first}.json"));
        let original = std::fs::read(&saved).unwrap();
        let serial_before = service.ids.load(Ordering::Acquire);
        let oversized = vec![b' '; MAX_CHECKPOINT + 1];
        assert!(matches!(
            service.persist_checkpoint(&first, &oversized),
            Err(PersistenceError::CheckpointTooLarge { bytes, limit })
                if bytes == MAX_CHECKPOINT + 1 && limit == MAX_CHECKPOINT
        ));
        assert_eq!(std::fs::read(&saved).unwrap(), original);
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
        assert_eq!(service.ids.load(Ordering::Acquire), serial_before);
        // The HTTP envelope fits the request limit, but its checkpoint cannot
        // be admitted and later persisted beyond this host's restore limit.
        let body = serde_json::to_vec(&json!({
            "checkpoint": {"oversized": "x".repeat(MAX_CHECKPOINT)}
        }))
        .unwrap();
        assert!(body.len() < MAX_BODY);
        assert_eq!(service.route("/api/import", &body).unwrap_err().0, 413);
        assert_eq!(service.sessions.lock().unwrap().len(), 1);
        assert_eq!(service.session_reservations.load(Ordering::Acquire), 1);
        assert_eq!(std::fs::read(&saved).unwrap(), original);
        assert_eq!(service.ids.load(Ordering::Acquire), serial_before);
        drop(service);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn saved_checkpoint_and_generation_capacity_preserve_existing_state() {
        let directory =
            std::env::temp_dir().join(format!("r4-native-service-capacity-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let service = Arc::new(Service::new(model(), Some(&directory)).unwrap());
        let first = id(&post(Arc::clone(&service), "/api/session", json!({})));
        let saved = directory.join(format!("{first}.json"));
        let original = std::fs::read(&saved).unwrap();
        let large = directory.join("capacity-fixture");
        std::fs::File::create(&large)
            .unwrap()
            .set_len(MAX_SAVED_BYTES)
            .unwrap();
        assert!(service.route("/api/session", b"{}").is_err());
        assert_eq!(std::fs::read(&saved).unwrap(), original);
        std::fs::remove_file(large).unwrap();
        for index in 1..MAX_SAVED_FILES {
            std::fs::write(directory.join(format!("fixture-{index}")), b"").unwrap();
        }
        assert!(service.route("/api/session", b"{}").is_err());
        assert_eq!(
            std::fs::read_dir(&directory).unwrap().count(),
            MAX_SAVED_FILES
        );
        // Updating an already reserved filename is allowed at the file cap.
        let slot = service.slot(&first).unwrap();
        service
            .persist(&first, &slot.session.lock().unwrap())
            .unwrap();
        service
            .generations
            .store(MAX_GENERATIONS, Ordering::Release);
        let error = service
            .route(
                "/api/generate",
                &serde_json::to_vec(&json!({"session_id":first,"prompt":"","max_tokens":1}))
                    .unwrap(),
            )
            .unwrap_err();
        assert_eq!(error.0, 429);
        // Control requests do not compete for the generation reservation.
        post(
            Arc::clone(&service),
            "/api/cancel",
            json!({"session_id":first}),
        );
        service.generations.store(0, Ordering::Release);
        drop(service);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
