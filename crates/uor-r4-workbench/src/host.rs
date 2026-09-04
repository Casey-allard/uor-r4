//! Loopback parent for the bounded native Four-fact workbench.
//!
//! HTTP threads perform transport admission and reserve lifecycle transitions
//! under one mutex.  A single controller thread is the sole owner of the child
//! process, its stdin, and the active private reply sequence.  The model
//! artifact is never opened by this module.

use crate::authority::frozen_accepted_binding;
use crate::base64::{decode_canonical, Base64Error};
use crate::http::{
    self, HttpAdmissionError, HttpRequest, Route, BODY_MAX_BYTES, HEADER_MAX_BYTES,
    MAX_CONNECTIONS, REQUEST_READ_DEADLINE_MS,
};
use crate::intake::{self, ValidatedConfiguration};
use crate::ipc::{self, AcceptedReply, ReplySequence};
use crate::launch::{self, VerifiedExecutable};
use crate::lifecycle::{Lifecycle, LifecycleError};
use crate::strict_json;
use crate::wire::{
    is_hex, AnswerRequest, ArtifactIdentity, CancelRequest, Capabilities, ErrorResponse,
    HistoricalReference, HostIdentity, IpcCommand, IpcLoad, IpcRequest, IpcRequestPayload,
    IpcResponse, JobKind, JobState, LoadRequest, ModelSnapshot, ModelState, Operation,
    ProgressStage, RawInput, ServiceError, ServiceErrorTag, UnloadRequest, CAPABILITIES_SCHEMA,
    CONFIGURED_MODEL_ID, CORE_INPUT_POLICY_MAX_BYTES, ERROR_SCHEMA, IPC_SCHEMA, OPERATION_ID,
    RAW_DECODED_MAX_BYTES, RAW_INPUT_SCHEMA, UINT53_MAX,
};
use crate::{BoxError, SERVICE_CONTRACT_SHA256};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddrV4, TcpListener, TcpStream};
use std::path::Path;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const LOAD_DEADLINE: Duration = Duration::from_millis(30_000);
const ANSWER_DEADLINE: Duration = Duration::from_millis(10_000);
const UNLOAD_DEADLINE: Duration = Duration::from_millis(4_000);
const REAP_POLL: Duration = Duration::from_millis(10);
const CONTROLLER_EVENT_CAP: usize = 64;
const STDERR_RETAIN_BYTES: usize = 65_536;
const STDERR_TRUNCATION: &[u8] = b"\n[worker stderr truncated]\n";

const HISTORICAL_BINARY_SHA256: &str =
    "d423d8d3c3acd2d1c6215c21206e1bec7583e4dd37e84f30f70f79e77c40d53f";
const HISTORICAL_QUALIFICATION_SHA256: &str =
    "61d29aa80e6bcd3d163b2ff2a6da4faab04414ea9f4284d80b798c4e46cf5369";

#[derive(Clone)]
struct Application {
    state: Arc<Mutex<Lifecycle>>,
    configuration: Arc<ValidatedConfiguration>,
    instance_id: Arc<str>,
    host: HostIdentity,
    artifact: ArtifactIdentity,
    host_available: bool,
    controller: mpsc::SyncSender<ControllerEvent>,
}

#[derive(Debug)]
enum ControllerCommand {
    Load {
        job_id: String,
        deadline_at: Instant,
    },
    Answer {
        job_id: String,
        input: RawInput,
        deadline_at: Instant,
    },
    Unload {
        job_id: String,
        deadline_at: Instant,
    },
    Stop {
        job_id: String,
    },
}

#[derive(Debug)]
enum ControllerEvent {
    Command(ControllerCommand),
    Reply {
        worker_generation: u64,
        response: IpcResponse,
    },
    ReaderClosed {
        worker_generation: u64,
        protocol_failure: bool,
    },
    WriterFailed {
        worker_generation: u64,
        job_id: String,
    },
    Stderr {
        worker_generation: u64,
        bytes: Vec<u8>,
        truncated: bool,
    },
    StderrReadFailed {
        worker_generation: u64,
    },
    Shutdown,
}

struct ActiveExchange {
    job_id: String,
    kind: JobKind,
    sequence: ReplySequence,
    started: Instant,
    deadline_at: Instant,
    unload_acknowledged: bool,
}

struct WorkerProcess {
    child: Child,
    writer: Option<mpsc::SyncSender<IpcRequest>>,
    writer_thread: Option<JoinHandle<()>>,
    generation: u64,
    active: Option<ActiveExchange>,
    stderr: StderrRetention,
    awaiting_reap: bool,
}

struct PendingLaunch {
    job_id: String,
    configuration_path: String,
    deadline_at: Instant,
    deadline_armed: bool,
    thread: JoinHandle<Result<Child, String>>,
}

struct UnadoptedChild {
    job_id: String,
    child: Child,
    cause: ServiceError,
}

#[derive(Default)]
struct StderrRetention {
    _bytes: Vec<u8>,
    _truncated: bool,
}

impl Drop for WorkerProcess {
    fn drop(&mut self) {
        force_reap_for_cleanup(&mut self.child);
        self.writer.take();
        if let Some(thread) = self.writer_thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for UnadoptedChild {
    fn drop(&mut self) {
        force_reap_for_cleanup(&mut self.child);
    }
}

struct Controller {
    state: Arc<Mutex<Lifecycle>>,
    configuration: Arc<ValidatedConfiguration>,
    executable: Arc<VerifiedExecutable>,
    events: mpsc::SyncSender<ControllerEvent>,
    pending_launch: Option<PendingLaunch>,
    unadopted_child: Option<UnadoptedChild>,
    worker: Option<WorkerProcess>,
}

impl Drop for Controller {
    fn drop(&mut self) {
        self.shutdown();
    }
}

struct ControllerGuard {
    sender: mpsc::SyncSender<ControllerEvent>,
    thread: Option<JoinHandle<()>>,
}

impl Drop for ControllerGuard {
    fn drop(&mut self) {
        let _ = self.sender.send(ControllerEvent::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

struct ConnectionGuard {
    active: Arc<AtomicUsize>,
}

#[derive(Debug, Clone)]
pub struct StartupError {
    pub error: ServiceError,
    rendered: String,
}

impl StartupError {
    fn new(tag: ServiceErrorTag, message: &'static str) -> Self {
        let error = service_error(tag, message);
        let rendered = serde_json::to_string(&error).unwrap_or_else(|_| {
            r#"{"tag":"WORKER_PROTOCOL_FAILURE","message":"Startup error serialization failed.","native":null}"#
                .to_owned()
        });
        Self { error, rendered }
    }

    fn bad_request(message: &'static str) -> Self {
        Self::new(ServiceErrorTag::BadRequest, message)
    }

    fn unsupported_runtime(message: &'static str) -> Self {
        Self::new(ServiceErrorTag::UnsupportedRuntime, message)
    }
}

impl std::fmt::Display for StartupError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.rendered)
    }
}

impl std::error::Error for StartupError {}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.active.fetch_sub(1, Ordering::AcqRel);
    }
}

#[derive(Debug, Clone)]
struct PublicError {
    status: u16,
    error: ServiceError,
    allow: Option<&'static str>,
}

impl PublicError {
    fn new(tag: ServiceErrorTag, message: impl Into<String>) -> Self {
        Self {
            status: status_for_tag(tag),
            error: service_error(tag, message),
            allow: None,
        }
    }

    fn from_http(error: HttpAdmissionError) -> Self {
        Self {
            status: error.status,
            error: error.as_service_error(),
            allow: error.allow,
        }
    }

    fn from_lifecycle(error: LifecycleError) -> Self {
        Self::new(error.service_tag(), error.to_string())
    }

    fn exact(error: ServiceError) -> Self {
        Self {
            status: status_for_tag(error.tag),
            error,
            allow: None,
        }
    }
}

/// Start the opt-in loopback service from one exact configuration identity.
///
/// Startup validates the executable, configuration, accepted evidence bundle,
/// and static assets before binding. `load_configuration` deliberately does
/// not open the configured model artifact.
pub fn serve(configuration_path: &Path, configuration_sha256: &str) -> Result<(), BoxError> {
    let executable = Arc::new(VerifiedExecutable::open_current().map_err(|_| {
        StartupError::unsupported_runtime(
            "Executable binding could not be adopted on this runtime.",
        )
    })?);
    let binding = frozen_accepted_binding().map_err(|_| {
        StartupError::unsupported_runtime("Embedded executable binding is not supported.")
    })?;
    let configuration = Arc::new(
        intake::load_configuration(
            configuration_path,
            configuration_sha256,
            executable.sha256(),
            &binding,
            SERVICE_CONTRACT_SHA256,
        )
        .map_err(|_| {
            StartupError::bad_request("Root configuration or static assets were rejected.")
        })?,
    );
    let instance_id = Arc::<str>::from(fresh_instance_id().map_err(|_| {
        StartupError::unsupported_runtime("A fresh service instance identity is unavailable.")
    })?);
    let host_available = configuration.host_acceptance.is_some();
    let lifecycle = if host_available {
        Lifecycle::new_unloaded(
            instance_id.to_string(),
            configuration.host.clone(),
            configuration.configured_artifact.clone(),
        )
        .map_err(|_| StartupError::bad_request("Accepted host configuration is invalid."))?
    } else {
        let message = configuration
            .host_acceptance_error
            .as_ref()
            .map(|error| format!("Current host qualification was not admitted: {error}"))
            .unwrap_or_else(|| "Current host has no accepted qualification.".to_owned());
        Lifecycle::new_unavailable(
            instance_id.to_string(),
            configuration.host.clone(),
            configuration.configured_artifact.clone(),
            service_error(ServiceErrorTag::UnavailableNativeQualification, message),
        )
        .map_err(|_| StartupError::bad_request("Discovery host configuration is invalid."))?
    };

    let address = SocketAddrV4::new(
        Ipv4Addr::LOCALHOST,
        u16::try_from(configuration.configuration.value.port)
            .map_err(|_| StartupError::bad_request("Configured port is outside u16."))?,
    );
    let listener = TcpListener::bind(address).map_err(|_| {
        StartupError::unsupported_runtime("The required loopback listener could not be bound.")
    })?;

    let state = Arc::new(Mutex::new(lifecycle));
    let (sender, receiver) = mpsc::sync_channel(CONTROLLER_EVENT_CAP);
    let controller_state = Arc::clone(&state);
    let controller_configuration = Arc::clone(&configuration);
    let controller_sender = sender.clone();
    let controller_thread = thread::Builder::new()
        .name("r4-workbench-controller".to_owned())
        .spawn(move || {
            Controller {
                state: controller_state,
                configuration: controller_configuration,
                executable,
                events: controller_sender,
                pending_launch: None,
                unadopted_child: None,
                worker: None,
            }
            .run(receiver);
        })
        .map_err(|_| {
            StartupError::unsupported_runtime("The private controller thread could not be started.")
        })?;
    let _controller_guard = ControllerGuard {
        sender: sender.clone(),
        thread: Some(controller_thread),
    };

    let application = Application {
        state,
        configuration: Arc::clone(&configuration),
        instance_id,
        host: configuration.host.clone(),
        artifact: configuration.configured_artifact.clone(),
        host_available,
        controller: sender,
    };
    let authority = Arc::<str>::from(format!("127.0.0.1:{}", address.port()));
    let active_connections = Arc::new(AtomicUsize::new(0));

    loop {
        let (stream, _) = match listener.accept() {
            Ok(connection) => connection,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => {
                return Err(StartupError::unsupported_runtime(
                    "The loopback listener failed after startup.",
                )
                .into())
            }
        };
        if active_connections
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                (count < MAX_CONNECTIONS).then_some(count + 1)
            })
            .is_err()
        {
            let _ = stream.shutdown(Shutdown::Both);
            continue;
        }

        let guard = ConnectionGuard {
            active: Arc::clone(&active_connections),
        };
        let application = application.clone();
        let authority = Arc::clone(&authority);
        let _ = thread::Builder::new()
            .name("r4-workbench-http".to_owned())
            .spawn(move || {
                let _guard = guard;
                handle_connection(stream, &application, &authority);
            });
    }
}

fn fresh_instance_id() -> Result<String, BoxError> {
    let mut random = File::open("/dev/urandom")?;
    let mut bytes = [0_u8; 16];
    random.read_exact(&mut bytes)?;
    let mut encoded = String::with_capacity(32);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut encoded, "{byte:02x}")?;
    }
    Ok(encoded)
}

fn lock_lifecycle(state: &Mutex<Lifecycle>) -> MutexGuard<'_, Lifecycle> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn handle_connection(mut stream: TcpStream, application: &Application, authority: &str) {
    let _ = stream.set_write_timeout(Some(Duration::from_millis(REQUEST_READ_DEADLINE_MS)));
    let response = match read_one_request(&mut stream)
        .map_err(PublicError::from_http)
        .and_then(|bytes| {
            let static_paths: Vec<&str> = application
                .configuration
                .assets
                .files
                .keys()
                .map(String::as_str)
                .collect();
            http::parse_request(&bytes, authority, &static_paths).map_err(PublicError::from_http)
        })
        .and_then(|request| dispatch(request, application))
    {
        Ok(response) => response,
        Err(error) => serialize_public_error(application, error),
    };
    if let Ok(response) = response {
        let _ = stream.write_all(&response);
        let _ = stream.flush();
    }
    let _ = stream.shutdown(Shutdown::Both);
}

fn read_one_request(stream: &mut TcpStream) -> Result<Vec<u8>, HttpAdmissionError> {
    let started = Instant::now();
    let maximum = HEADER_MAX_BYTES
        .checked_add(BODY_MAX_BYTES)
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| local_http_error("HTTP request limit overflow"))?;
    let mut bytes = Vec::with_capacity(maximum.min(24_576));
    let mut chunk = [0_u8; 4_096];

    loop {
        if request_is_complete(&bytes) || bytes.len() >= maximum {
            break;
        }
        let remaining_time = Duration::from_millis(REQUEST_READ_DEADLINE_MS)
            .checked_sub(started.elapsed())
            .ok_or_else(|| local_http_error("HTTP request read deadline exceeded"))?;
        if remaining_time.is_zero() {
            return Err(local_http_error("HTTP request read deadline exceeded"));
        }
        stream
            .set_read_timeout(Some(remaining_time))
            .map_err(|_| local_http_error("HTTP request timeout could not be applied"))?;
        let wanted = chunk.len().min(maximum.saturating_sub(bytes.len()));
        let count = stream
            .read(&mut chunk[..wanted])
            .map_err(|_| local_http_error("HTTP request could not be read"))?;
        if count == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..count]);
    }

    // Catch bytes which were already pipelined. Future writes are immaterial:
    // every accepted connection is closed after this one request.
    if request_is_complete(&bytes) && bytes.len() < maximum {
        if stream.set_nonblocking(true).is_ok() {
            let mut trailing = [0_u8; 1];
            if stream.peek(&mut trailing).is_ok_and(|count| count != 0) {
                bytes.push(trailing[0]);
            }
            let _ = stream.set_nonblocking(false);
        }
    }
    Ok(bytes)
}

fn local_http_error(message: &'static str) -> HttpAdmissionError {
    HttpAdmissionError {
        tag: ServiceErrorTag::BadRequest,
        status: 400,
        message,
        allow: None,
    }
}

/// Return true once enough bytes are present for the declared request. Invalid
/// framing is considered complete as soon as the header is available so the
/// canonical parser can return the precise transport error without reading an
/// untrusted body.
fn request_is_complete(bytes: &[u8]) -> bool {
    let Some(header_end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") else {
        return bytes.len() > HEADER_MAX_BYTES;
    };
    let body_start = match header_end.checked_add(4) {
        Some(value) => value,
        None => return true,
    };
    if body_start > HEADER_MAX_BYTES {
        return true;
    }
    let Ok(header) = std::str::from_utf8(&bytes[..header_end]) else {
        return true;
    };
    let mut lines = header.split("\r\n");
    let method = lines
        .next()
        .and_then(|line| line.split(' ').next())
        .unwrap_or_default();
    let mut content_length = None;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            return true;
        };
        if name.eq_ignore_ascii_case("transfer-encoding") {
            return true;
        }
        if name.eq_ignore_ascii_case("content-length") {
            if content_length.is_some() {
                return true;
            }
            let value = value.trim_matches(&[' ', '\t'][..]);
            let Ok(length) = value.parse::<usize>() else {
                return true;
            };
            if length > BODY_MAX_BYTES {
                return true;
            }
            content_length = Some(length);
        }
    }
    let body_length = match content_length {
        Some(value) => value,
        None if method == "POST" => return true,
        None => 0,
    };
    body_start
        .checked_add(body_length)
        .is_none_or(|expected| bytes.len() >= expected)
}

fn dispatch(request: HttpRequest, application: &Application) -> Result<Vec<u8>, PublicError> {
    match request.route {
        Route::Capabilities => {
            let body = capabilities(application)?;
            serialize_json(200, &body, None)
        }
        Route::Model => {
            let model = lock_lifecycle(&application.state).model_snapshot();
            serialize_json(200, &model, None)
        }
        Route::Load => admit_load(&request.body, application),
        Route::Unload => admit_unload(&request.body, application),
        Route::Requests => admit_answer(&request.body, application),
        Route::Job { job_id } => {
            let job = lock_lifecycle(&application.state)
                .job(&job_id)
                .ok_or_else(|| {
                    PublicError::new(ServiceErrorTag::JobNotFound, "Job was not found.")
                })?;
            serialize_json(200, &job, None)
        }
        Route::Cancel { job_id } => admit_cancel(&request.body, &job_id, application),
        Route::Static { manifest_path } => {
            let asset = application
                .configuration
                .assets
                .files
                .get(&manifest_path)
                .ok_or_else(|| {
                    PublicError::new(ServiceErrorTag::NotFound, "Asset was not found.")
                })?;
            http::serialize_static_response(&asset.mime, &asset.bytes)
                .map_err(PublicError::from_http)
        }
    }
}

fn capabilities(application: &Application) -> Result<Capabilities, PublicError> {
    let model = lock_lifecycle(&application.state).model_snapshot();
    let (enabled, unavailable_reason) = operation_availability(&model);
    let capabilities = Capabilities {
        schema: CAPABILITIES_SCHEMA.to_owned(),
        instance_id: application.instance_id.to_string(),
        revision: model.revision,
        provider: "native".to_owned(),
        execution: "cpu-floating-point-research-reference".to_owned(),
        host: application.host.clone(),
        configured_artifact: application.artifact.clone(),
        model_state: model.state,
        operations: vec![Operation {
            id: OPERATION_ID.to_owned(),
            input_schema: RAW_INPUT_SCHEMA.to_owned(),
            result_schemas: vec![
                "uor-r4.text-binding-result/1".to_owned(),
                "uor-r4.text-to-clauses-result/1".to_owned(),
            ],
            enabled,
            unavailable_reason,
            stateless: true,
            input_policy_max_bytes: CORE_INPUT_POLICY_MAX_BYTES,
            decoded_transport_max_bytes: RAW_DECODED_MAX_BYTES as u64,
            general_generation: false,
            general_context: false,
            coding: false,
            final_integer_kernel: false,
        }],
        historical_reference: HistoricalReference {
            issue: "1102".to_owned(),
            terminal: "NATIVE_REFERENCE_PRESERVED".to_owned(),
            binary_sha256: HISTORICAL_BINARY_SHA256.to_owned(),
            qualification_sha256: HISTORICAL_QUALIFICATION_SHA256.to_owned(),
            scope: "known-authoring-four-fact-reference".to_owned(),
            applies_to_current_host: false,
        },
        active_job_id: model.active_job_id,
        last_job_id: model.last_job_id,
    };
    capabilities.validate().map_err(|error| {
        PublicError::new(ServiceErrorTag::WorkerProtocolFailure, error.to_string())
    })?;
    Ok(capabilities)
}

fn operation_availability(model: &ModelSnapshot) -> (bool, Option<ServiceError>) {
    match model.state {
        ModelState::Ready => (true, None),
        ModelState::Loading
        | ModelState::Running
        | ModelState::Stopping
        | ModelState::Unloading => (
            false,
            Some(service_error(
                ServiceErrorTag::Busy,
                "The single model lifecycle slot is busy.",
            )),
        ),
        ModelState::Unloaded => (
            false,
            Some(service_error(
                ServiceErrorTag::NotReady,
                "The native research reference is not loaded.",
            )),
        ),
        ModelState::Unavailable | ModelState::Error => (
            false,
            Some(model.error.clone().unwrap_or_else(|| {
                service_error(
                    ServiceErrorTag::UnavailableNativeQualification,
                    "The native research reference is unavailable.",
                )
            })),
        ),
    }
}

fn admit_load(body: &[u8], application: &Application) -> Result<Vec<u8>, PublicError> {
    let request: LoadRequest = parse_body(body)?;
    require_schema(&request.schema, crate::wire::LOAD_SCHEMA)?;
    require_request_instance_shape(&request.instance_id)?;
    require_instance(&request.instance_id, application)?;
    require_model(&request.model_id)?;

    let deadline_at = Instant::now() + LOAD_DEADLINE;
    let response = {
        let mut state = lock_lifecycle(&application.state);
        require_host_available(application, &state.model_snapshot())?;
        let job = state.admit_load().map_err(PublicError::from_lifecycle)?;
        accepted_job_response_locked(
            application,
            &mut state,
            &job.job_id,
            Some(ControllerCommand::Load {
                job_id: job.job_id.clone(),
                deadline_at,
            }),
        )?
    };
    Ok(response)
}

fn admit_unload(body: &[u8], application: &Application) -> Result<Vec<u8>, PublicError> {
    let request: UnloadRequest = parse_body(body)?;
    require_schema(&request.schema, crate::wire::UNLOAD_SCHEMA)?;
    require_request_instance_shape(&request.instance_id)?;
    require_instance(&request.instance_id, application)?;
    require_model(&request.model_id)?;
    require_uint53_shape(request.expected_generation)?;
    let deadline_at = Instant::now() + UNLOAD_DEADLINE;
    let response = {
        let mut state = lock_lifecycle(&application.state);
        let job = state
            .admit_unload(request.expected_generation)
            .map_err(PublicError::from_lifecycle)?;
        accepted_job_response_locked(
            application,
            &mut state,
            &job.job_id,
            Some(ControllerCommand::Unload {
                job_id: job.job_id.clone(),
                deadline_at,
            }),
        )?
    };
    Ok(response)
}

fn admit_answer(body: &[u8], application: &Application) -> Result<Vec<u8>, PublicError> {
    let request: AnswerRequest = parse_body(body)?;
    require_schema(&request.schema, crate::wire::REQUEST_SCHEMA)?;
    require_request_instance_shape(&request.instance_id)?;
    require_instance(&request.instance_id, application)?;
    require_model(&request.model_id)?;
    if request.operation != OPERATION_ID {
        return Err(PublicError::new(
            ServiceErrorTag::UnsupportedOperation,
            "Requested operation is not supported.",
        ));
    }
    if request.input.schema != RAW_INPUT_SCHEMA {
        return Err(PublicError::new(
            ServiceErrorTag::UnsupportedSchema,
            "Raw input schema is not supported.",
        ));
    }
    require_uint53_shape(request.expected_generation)?;

    let deadline_at = Instant::now() + ANSWER_DEADLINE;
    let response = {
        let mut state = lock_lifecycle(&application.state);
        let model = state.model_snapshot();
        require_host_available(application, &model)?;
        if state.active_job().is_some() || model.state == ModelState::Stopping {
            return Err(PublicError::new(
                ServiceErrorTag::Busy,
                "The single model lifecycle slot is busy.",
            ));
        }
        if model.state != ModelState::Ready {
            return Err(PublicError::new(
                ServiceErrorTag::NotReady,
                "The native research reference is not ready.",
            ));
        }
        if request.expected_generation != model.model_generation {
            return Err(PublicError::new(
                ServiceErrorTag::StaleModel,
                format!(
                    "Expected model generation {}; current generation is {}.",
                    request.expected_generation, model.model_generation
                ),
            ));
        }
        if request.input.encoding != "base64" {
            return Err(PublicError::new(
                ServiceErrorTag::InvalidBase64,
                "Raw input encoding must be canonical base64.",
            ));
        }
        let decoded = decode_canonical(&request.input.bytes_b64, RAW_DECODED_MAX_BYTES)
            .map_err(map_base64_error)?;
        let raw_sha256 = intake::sha256(&decoded);
        let job = state
            .admit_answer(request.expected_generation, raw_sha256)
            .map_err(PublicError::from_lifecycle)?;
        accepted_job_response_locked(
            application,
            &mut state,
            &job.job_id,
            Some(ControllerCommand::Answer {
                job_id: job.job_id.clone(),
                input: request.input,
                deadline_at,
            }),
        )?
    };
    Ok(response)
}

fn admit_cancel(
    body: &[u8],
    job_id: &str,
    application: &Application,
) -> Result<Vec<u8>, PublicError> {
    let request: CancelRequest = parse_body(body)?;
    require_schema(&request.schema, crate::wire::CANCEL_SCHEMA)?;
    require_request_instance_shape(&request.instance_id)?;
    require_instance(&request.instance_id, application)?;
    let response = {
        let mut state = lock_lifecycle(&application.state);
        let before = state.job(job_id);
        let job = state
            .request_cancel(job_id)
            .map_err(PublicError::from_lifecycle)?;
        let signal = before.is_some_and(|prior| prior.state != JobState::Stopping)
            && job.state == JobState::Stopping;
        let command = signal.then(|| ControllerCommand::Stop {
            job_id: job.job_id.clone(),
        });
        accepted_job_response_locked(application, &mut state, &job.job_id, command)?
    };
    Ok(response)
}

fn accepted_job_response_locked(
    application: &Application,
    state: &mut Lifecycle,
    job_id: &str,
    command: Option<ControllerCommand>,
) -> Result<Vec<u8>, PublicError> {
    if let Some(command) = command {
        match application
            .controller
            .try_send(ControllerEvent::Command(command))
        {
            Ok(()) => {}
            Err(mpsc::TrySendError::Full(_)) => {
                let cause = service_error(
                    ServiceErrorTag::WorkerFailure,
                    "The bounded private controller queue could not accept the admitted operation.",
                );
                let _ = state.worker_failure(job_id, cause);
            }
            Err(mpsc::TrySendError::Disconnected(_)) => {
                let cause = service_error(
                    ServiceErrorTag::WorkerFailure,
                    "The private worker controller disconnected from the admitted operation.",
                );
                let _ = state.worker_failure(job_id, cause);
                let _ = state.note_termination_unconfirmed(job_id);
            }
        }
    }
    let latest = state.job(job_id).ok_or_else(|| {
        PublicError::new(
            ServiceErrorTag::WorkerProtocolFailure,
            "The admitted job snapshot could not be retained for its response.",
        )
    })?;
    serialize_json(202, &latest, None)
}

fn parse_body<T: DeserializeOwned>(body: &[u8]) -> Result<T, PublicError> {
    strict_json::from_slice(body).map_err(|_| {
        PublicError::new(
            ServiceErrorTag::BadRequest,
            "Request JSON does not match the exact route schema.",
        )
    })
}

fn require_schema(actual: &str, expected: &str) -> Result<(), PublicError> {
    if actual == expected {
        Ok(())
    } else {
        Err(PublicError::new(
            ServiceErrorTag::UnsupportedSchema,
            "Request schema is not supported.",
        ))
    }
}

fn require_request_instance_shape(actual: &str) -> Result<(), PublicError> {
    if is_hex(actual, 32) {
        Ok(())
    } else {
        Err(PublicError::new(
            ServiceErrorTag::BadRequest,
            "Request instance_id is not lowercase hex32.",
        ))
    }
}

fn require_uint53_shape(actual: u64) -> Result<(), PublicError> {
    if actual <= UINT53_MAX {
        Ok(())
    } else {
        Err(PublicError::new(
            ServiceErrorTag::BadRequest,
            "Request integer exceeds the uint53 wire limit.",
        ))
    }
}

fn require_instance(actual: &str, application: &Application) -> Result<(), PublicError> {
    if actual == application.instance_id.as_ref() {
        Ok(())
    } else {
        Err(PublicError::new(
            ServiceErrorTag::StaleInstance,
            "Request names a different service instance.",
        ))
    }
}

fn require_model(actual: &str) -> Result<(), PublicError> {
    if actual == CONFIGURED_MODEL_ID {
        Ok(())
    } else {
        Err(PublicError::new(
            ServiceErrorTag::ModelNotFound,
            "Requested model is not configured.",
        ))
    }
}

fn require_host_available(
    application: &Application,
    model: &ModelSnapshot,
) -> Result<(), PublicError> {
    if application.host_available {
        Ok(())
    } else {
        Err(PublicError::exact(model.error.clone().unwrap_or_else(
            || {
                service_error(
                    ServiceErrorTag::UnavailableNativeQualification,
                    "Current host has no accepted qualification.",
                )
            },
        )))
    }
}

fn map_base64_error(error: Base64Error) -> PublicError {
    match error {
        Base64Error::InvalidEncoding => PublicError::new(
            ServiceErrorTag::InvalidBase64,
            "Raw input is not canonical padded standard base64.",
        ),
        Base64Error::DecodedTooLarge => PublicError::new(
            ServiceErrorTag::RawInputTooLarge,
            "Decoded raw input exceeds the transport limit.",
        ),
    }
}

fn serialize_json<T: Serialize>(
    status: u16,
    value: &T,
    allow: Option<&'static str>,
) -> Result<Vec<u8>, PublicError> {
    let body = serde_json::to_vec(value).map_err(|_| {
        PublicError::new(
            ServiceErrorTag::WorkerProtocolFailure,
            "Response serialization failed.",
        )
    })?;
    http::serialize_json_response(status, &body, allow).map_err(PublicError::from_http)
}

fn serialize_public_error(
    application: &Application,
    error: PublicError,
) -> Result<Vec<u8>, HttpAdmissionError> {
    let revision = lock_lifecycle(&application.state).revision();
    let response = ErrorResponse {
        schema: ERROR_SCHEMA.to_owned(),
        instance_id: application.instance_id.to_string(),
        revision,
        error: error.error,
    };
    let body = serde_json::to_vec(&response).unwrap_or_else(|_| {
        br#"{"schema":"uor-r4.workbench-error/1","instance_id":"00000000000000000000000000000000","revision":0,"error":{"tag":"WORKER_PROTOCOL_FAILURE","message":"Response serialization failed.","native":null}}"#.to_vec()
    });
    http::serialize_json_response(error.status, &body, error.allow)
}

fn status_for_tag(tag: ServiceErrorTag) -> u16 {
    match tag {
        ServiceErrorTag::BadRequest
        | ServiceErrorTag::UnsupportedSchema
        | ServiceErrorTag::UnsupportedOperation
        | ServiceErrorTag::InvalidBase64 => 400,
        ServiceErrorTag::OriginRejected => 403,
        ServiceErrorTag::NotFound
        | ServiceErrorTag::ModelNotFound
        | ServiceErrorTag::JobNotFound => 404,
        ServiceErrorTag::MethodNotAllowed => 405,
        ServiceErrorTag::StaleInstance
        | ServiceErrorTag::StaleModel
        | ServiceErrorTag::Busy
        | ServiceErrorTag::NotReady
        | ServiceErrorTag::AlreadyLoaded
        | ServiceErrorTag::AlreadyUnloaded
        | ServiceErrorTag::AlreadyTerminal
        | ServiceErrorTag::NotCancellable => 409,
        ServiceErrorTag::BodyTooLarge | ServiceErrorTag::RawInputTooLarge => 413,
        ServiceErrorTag::UnsupportedMediaType => 415,
        ServiceErrorTag::HostRejected => 421,
        ServiceErrorTag::UnavailableNativeQualification
        | ServiceErrorTag::UnsupportedRuntime
        | ServiceErrorTag::UnavailableArtifact
        | ServiceErrorTag::ArtifactRejected => 503,
        ServiceErrorTag::NativeFailure
        | ServiceErrorTag::WorkerFailure
        | ServiceErrorTag::WorkerProtocolFailure
        | ServiceErrorTag::TerminationUnconfirmed => 500,
        ServiceErrorTag::DeadlineExceeded => 504,
    }
}

fn service_error(tag: ServiceErrorTag, message: impl Into<String>) -> ServiceError {
    let mut bounded = String::with_capacity(512);
    for character in message.into().chars().filter(|value| !value.is_control()) {
        if bounded.len() + character.len_utf8() > 512 {
            break;
        }
        bounded.push(character);
    }
    ServiceError {
        tag,
        message: bounded,
        native: None,
    }
}

impl Controller {
    fn run(mut self, receiver: mpsc::Receiver<ControllerEvent>) {
        loop {
            let received = match self.next_wait() {
                Some(wait) => receiver.recv_timeout(wait),
                None => receiver
                    .recv()
                    .map_err(|_| mpsc::RecvTimeoutError::Disconnected),
            };
            let event = match received {
                Ok(event) => event,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    self.handle_due_timeouts();
                    continue;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    self.shutdown();
                    break;
                }
            };
            // A continuously readable channel must not postpone the sole
            // active admission's wall-clock deadline. Arbitrate time before
            // any newly received reply, progress, command, or launch result.
            self.handle_due_timeouts();
            match event {
                ControllerEvent::Command(command) => self.handle_command(command),
                ControllerEvent::Reply {
                    worker_generation,
                    response,
                } => self.handle_reply(worker_generation, response),
                ControllerEvent::ReaderClosed {
                    worker_generation,
                    protocol_failure,
                } => self.handle_reader_closed(worker_generation, protocol_failure),
                ControllerEvent::WriterFailed {
                    worker_generation,
                    job_id,
                } => self.handle_writer_failed(worker_generation, &job_id),
                ControllerEvent::Stderr {
                    worker_generation,
                    mut bytes,
                    truncated,
                } => {
                    if let Some(worker) = self
                        .worker
                        .as_mut()
                        .filter(|worker| worker.generation == worker_generation)
                    {
                        bytes.truncate(STDERR_RETAIN_BYTES);
                        worker.stderr = StderrRetention {
                            _bytes: bytes,
                            _truncated: truncated,
                        };
                    }
                }
                ControllerEvent::StderrReadFailed { worker_generation } => {
                    self.handle_stderr_read_failed(worker_generation)
                }
                ControllerEvent::Shutdown => {
                    self.shutdown();
                    break;
                }
            }
        }
    }

    fn next_wait(&self) -> Option<Duration> {
        let now = Instant::now();
        let mut wait = None;
        let mut include = |candidate: Duration| {
            wait = Some(wait.map_or(candidate, |current: Duration| current.min(candidate)));
        };
        if let Some(pending) = self
            .pending_launch
            .as_ref()
            .filter(|pending| pending.deadline_armed)
        {
            include(pending.deadline_at.saturating_duration_since(now));
        }
        if self.pending_launch.is_some() {
            include(REAP_POLL);
        }
        if let Some(exchange) = self
            .worker
            .as_ref()
            .and_then(|worker| worker.active.as_ref())
        {
            include(exchange.deadline_at.saturating_duration_since(now));
        }
        if self.worker.is_some() || self.unadopted_child.is_some() {
            include(REAP_POLL);
        }
        wait
    }

    fn handle_due_timeouts(&mut self) {
        let now = Instant::now();
        let pending_job = self.pending_launch.as_mut().and_then(|pending| {
            if pending.deadline_armed && now >= pending.deadline_at {
                pending.deadline_armed = false;
                Some(pending.job_id.clone())
            } else {
                None
            }
        });
        if let Some(job_id) = pending_job {
            let mut state = lock_lifecycle(&self.state);
            if state.active_job_id() == Some(job_id.as_str()) {
                let _ = state.deadline(&job_id);
            }
        }
        self.poll_pending_launch();
        let stopping_job = {
            let state = lock_lifecycle(&self.state);
            state
                .active_job()
                .filter(|job| job.state == JobState::Stopping)
                .map(|job| job.job_id)
        };
        if let Some(job_id) = stopping_job {
            self.stop_reserved(&job_id);
        }
        let active_deadline = self.worker.as_ref().and_then(|worker| {
            worker.active.as_ref().and_then(|exchange| {
                (now >= exchange.deadline_at)
                    .then_some((worker.generation, exchange.job_id.clone()))
            })
        });
        if let Some((generation, job_id)) = active_deadline {
            self.handle_deadline(generation, &job_id);
        }
        if self
            .worker
            .as_ref()
            .is_some_and(|worker| worker.awaiting_reap)
        {
            self.poll_reap();
        }
        self.poll_unadopted_child();
    }

    fn handle_command(&mut self, command: ControllerCommand) {
        match command {
            ControllerCommand::Load {
                job_id,
                deadline_at,
            } => self.start_load(&job_id, deadline_at),
            ControllerCommand::Answer {
                job_id,
                input,
                deadline_at,
            } => self.start_existing(
                &job_id,
                JobKind::Answer,
                IpcRequestPayload::Answer(input),
                deadline_at,
            ),
            ControllerCommand::Unload {
                job_id,
                deadline_at,
            } => self.start_existing(
                &job_id,
                JobKind::Unload,
                IpcRequestPayload::Empty(()),
                deadline_at,
            ),
            ControllerCommand::Stop { job_id } => self.stop_reserved(&job_id),
        }
    }

    fn start_load(&mut self, job_id: &str, deadline_at: Instant) {
        if self.worker.is_some() || self.pending_launch.is_some() || self.unadopted_child.is_some()
        {
            self.fail_active(
                job_id,
                service_error(
                    ServiceErrorTag::WorkerProtocolFailure,
                    "A worker or launcher existed before an admitted load.",
                ),
            );
            return;
        }
        let configuration_path = match self.configuration.configuration.path.to_str() {
            Some(path) => path.to_owned(),
            None => {
                self.fail_active(
                    job_id,
                    service_error(
                        ServiceErrorTag::WorkerProtocolFailure,
                        "Adopted configuration path is not UTF-8.",
                    ),
                );
                return;
            }
        };
        {
            let mut state = lock_lifecycle(&self.state);
            if state.active_job_id() != Some(job_id)
                || state.active_job_kind() != Some(JobKind::Load)
                || state.worker_spawned_for_active_job()
            {
                return;
            }
            if Instant::now() >= deadline_at {
                let _ = state.deadline(job_id);
                return;
            }
            if let Err(error) = state.note_load_launch_pending(job_id) {
                let cause = service_error(error.service_tag(), error.to_string());
                let _ = state.worker_failure(job_id, cause);
                return;
            }
        }

        let executable = Arc::clone(&self.executable);
        let launch = thread::Builder::new()
            .name("r4-workbench-launch".to_owned())
            .spawn(move || {
                executable
                    .spawn_worker()
                    .map_err(|_| "accepted private worker spawn failed".to_owned())
            });
        match launch {
            Ok(thread) => {
                self.pending_launch = Some(PendingLaunch {
                    job_id: job_id.to_owned(),
                    configuration_path,
                    deadline_at,
                    deadline_armed: true,
                    thread,
                });
            }
            Err(_) => {
                let cause = service_error(
                    ServiceErrorTag::WorkerFailure,
                    "The bounded private worker launcher could not be started.",
                );
                let mut state = lock_lifecycle(&self.state);
                if Instant::now() >= deadline_at {
                    let _ = state.deadline(job_id);
                }
                let _ = state.finish_load_launch_without_child(job_id, cause);
            }
        }
    }

    fn poll_pending_launch(&mut self) {
        if !self
            .pending_launch
            .as_ref()
            .is_some_and(|pending| pending.thread.is_finished())
        {
            return;
        }
        let Some(pending) = self.pending_launch.take() else {
            return;
        };
        let job_id = pending.job_id.clone();
        let launch_result = match pending.thread.join() {
            Ok(result) => result,
            Err(_) => {
                let cause = service_error(
                    ServiceErrorTag::WorkerFailure,
                    "Private worker launch completion could not be confirmed.",
                );
                let mut state = lock_lifecycle(&self.state);
                let _ = state.worker_failure(&job_id, cause);
                let _ = state.note_termination_unconfirmed(&job_id);
                return;
            }
        };
        let child = match launch_result {
            Ok(child) => child,
            Err(_) => {
                let cause = service_error(
                    ServiceErrorTag::WorkerFailure,
                    "The accepted private worker could not be started.",
                );
                let mut state = lock_lifecycle(&self.state);
                if Instant::now() >= pending.deadline_at {
                    let _ = state.deadline(&job_id);
                }
                let _ = state.finish_load_launch_without_child(&job_id, cause);
                return;
            }
        };

        let mut offered_child = Some(child);
        let adopted = {
            let mut state = lock_lifecycle(&self.state);
            state.spawn_load_worker(&job_id, || {
                offered_child
                    .take()
                    .ok_or("returned child was already consumed")
            })
        };
        let (generation, mut child) = match adopted {
            Ok(Ok(adopted)) => adopted,
            _ => {
                let Some(mut child) = offered_child.take() else {
                    return;
                };
                let cause = service_error(
                    ServiceErrorTag::WorkerProtocolFailure,
                    "Returned worker could not be bound to its admitted load.",
                );
                let mut state = lock_lifecycle(&self.state);
                let _ = state.worker_failure(&job_id, cause.clone());
                drop(state);
                if launch::terminate_owned(&mut child).is_ok() {
                    let _ = lock_lifecycle(&self.state)
                        .finish_load_launch_without_child(&job_id, cause);
                } else {
                    let _ = lock_lifecycle(&self.state).note_termination_unconfirmed(&job_id);
                    self.unadopted_child = Some(UnadoptedChild {
                        job_id,
                        child,
                        cause,
                    });
                }
                return;
            }
        };
        let stdin = child.stdin.take();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        self.worker = Some(WorkerProcess {
            child,
            writer: None,
            writer_thread: None,
            generation,
            active: None,
            stderr: StderrRetention::default(),
            awaiting_reap: false,
        });
        let should_stop = {
            let mut state = lock_lifecycle(&self.state);
            if Instant::now() >= pending.deadline_at {
                let _ = state.deadline(&job_id);
            }
            state
                .job(&job_id)
                .is_some_and(|job| job.state == JobState::Stopping)
        };
        if should_stop {
            self.stop_reserved(&job_id);
            return;
        }
        let (Some(stdin), Some(stdout), Some(stderr)) = (stdin, stdout, stderr) else {
            self.fail_active(
                &job_id,
                service_error(
                    ServiceErrorTag::WorkerProtocolFailure,
                    "Private worker pipes were not created.",
                ),
            );
            return;
        };
        let Some((writer, writer_thread)) =
            start_stdin_writer(stdin, generation, self.events.clone())
        else {
            self.fail_active(
                &job_id,
                service_error(
                    ServiceErrorTag::WorkerFailure,
                    "Private worker writer thread could not be started.",
                ),
            );
            return;
        };
        if let Some(worker) = self.worker.as_mut() {
            worker.writer = Some(writer);
            worker.writer_thread = Some(writer_thread);
        }
        let reply_reader = start_stdout_reader(stdout, generation, self.events.clone());
        let stderr_reader = start_stderr_reader(stderr, generation, self.events.clone());
        if !reply_reader || !stderr_reader {
            self.fail_active(
                &job_id,
                service_error(
                    ServiceErrorTag::WorkerFailure,
                    "Private worker drain threads could not be started.",
                ),
            );
            return;
        }
        let request = IpcRequest {
            schema: IPC_SCHEMA.to_owned(),
            instance_id: self.configuration_instance_id(),
            job_id: job_id.clone(),
            worker_generation: generation,
            command: IpcCommand::Load,
            payload: IpcRequestPayload::Load(IpcLoad {
                configuration_path: pending.configuration_path,
                configuration_sha256: self.configuration.configuration.sha256.clone(),
            }),
        };
        self.dispatch_exchange(request, JobKind::Load, pending.deadline_at);
    }

    fn start_existing(
        &mut self,
        job_id: &str,
        kind: JobKind,
        payload: IpcRequestPayload,
        deadline_at: Instant,
    ) {
        let generation = {
            let state = lock_lifecycle(&self.state);
            if state.active_job_id() != Some(job_id) || state.active_job_kind() != Some(kind) {
                return;
            }
            match state.current_worker_generation() {
                Some(generation) => generation,
                None => {
                    drop(state);
                    self.fail_active(
                        job_id,
                        service_error(
                            ServiceErrorTag::WorkerProtocolFailure,
                            "Admitted operation has no owned worker.",
                        ),
                    );
                    return;
                }
            }
        };
        if !self
            .worker
            .as_ref()
            .is_some_and(|worker| worker.generation == generation && worker.active.is_none())
        {
            self.fail_active(
                job_id,
                service_error(
                    ServiceErrorTag::WorkerProtocolFailure,
                    "Owned worker does not match the admitted lifecycle.",
                ),
            );
            return;
        }
        let request = IpcRequest {
            schema: IPC_SCHEMA.to_owned(),
            instance_id: self.configuration_instance_id(),
            job_id: job_id.to_owned(),
            worker_generation: generation,
            command: match kind {
                JobKind::Answer => IpcCommand::Answer,
                JobKind::Unload => IpcCommand::Unload,
                JobKind::Load => return,
            },
            payload,
        };
        self.dispatch_exchange(request, kind, deadline_at);
    }

    fn configuration_instance_id(&self) -> String {
        lock_lifecycle(&self.state).model_snapshot().instance_id
    }

    fn dispatch_exchange(&mut self, request: IpcRequest, kind: JobKind, deadline_at: Instant) {
        let job_id = request.job_id.clone();
        let generation = request.worker_generation;
        let sequence = match ReplySequence::new(
            &request,
            self.configuration.configured_artifact.artifact_bytes,
        ) {
            Ok(sequence) => sequence,
            Err(_) => {
                self.fail_active(
                    &job_id,
                    service_error(
                        ServiceErrorTag::WorkerProtocolFailure,
                        "Private request sequence was rejected.",
                    ),
                );
                return;
            }
        };
        let started = Instant::now();
        let state_handle = Arc::clone(&self.state);
        let mut state = lock_lifecycle(&state_handle);
        let current = state.job(&job_id);
        if !current.as_ref().is_some_and(|job| {
            job.kind == kind
                && !matches!(
                    job.state,
                    JobState::Stopping
                        | JobState::Completed
                        | JobState::Cancelled
                        | JobState::Failed
                )
        }) {
            let stopping = current.is_some_and(|job| job.state == JobState::Stopping);
            drop(state);
            if stopping {
                self.stop_reserved(&job_id);
            }
            return;
        }
        if Instant::now() >= deadline_at {
            let _ = state.deadline(&job_id);
            drop(state);
            self.stop_reserved(&job_id);
            return;
        }
        let Some(worker) = self
            .worker
            .as_mut()
            .filter(|worker| worker.generation == generation && worker.active.is_none())
        else {
            drop(state);
            self.fail_active(
                &job_id,
                service_error(ServiceErrorTag::WorkerFailure, "Owned worker disappeared."),
            );
            return;
        };
        worker.active = Some(ActiveExchange {
            job_id: job_id.clone(),
            kind,
            sequence,
            started,
            deadline_at,
            unload_acknowledged: false,
        });
        let Some(writer) = worker.writer.as_ref() else {
            drop(state);
            self.fail_active(
                &job_id,
                service_error(
                    ServiceErrorTag::WorkerProtocolFailure,
                    "Owned worker writer disappeared.",
                ),
            );
            return;
        };
        if writer.try_send(request).is_err() {
            drop(state);
            self.fail_active(
                &job_id,
                service_error(
                    ServiceErrorTag::WorkerFailure,
                    "Private worker command could not be handed to its bounded writer.",
                ),
            );
            return;
        }
        if state.mark_dispatched(&job_id).is_err() {
            drop(state);
            self.fail_active(
                &job_id,
                service_error(
                    ServiceErrorTag::WorkerProtocolFailure,
                    "Dispatched command did not match the active lifecycle.",
                ),
            );
            return;
        }
        drop(state);
    }

    fn handle_reply(&mut self, generation: u64, response: IpcResponse) {
        let Some(worker) = self
            .worker
            .as_mut()
            .filter(|worker| worker.generation == generation)
        else {
            return;
        };
        let Some(exchange) = worker.active.as_mut() else {
            self.fail_idle(service_error(
                ServiceErrorTag::WorkerProtocolFailure,
                "Private worker sent output without an active command.",
            ));
            return;
        };
        let job_id = exchange.job_id.clone();
        if lock_lifecycle(&self.state)
            .job(&job_id)
            .is_some_and(|job| job.state == JobState::Stopping)
        {
            exchange.sequence.mark_parent_stop();
        }
        let accepted = match exchange.sequence.accept(response) {
            Ok(accepted) => accepted,
            Err(_) => {
                self.fail_active(
                    &job_id,
                    service_error(
                        ServiceErrorTag::WorkerProtocolFailure,
                        "Private worker reply violated the active sequence.",
                    ),
                );
                return;
            }
        };
        let elapsed = elapsed_ms(exchange.started);
        match accepted {
            AcceptedReply::DiscardedAfterStop => {
                self.stop_reserved(&job_id);
            }
            AcceptedReply::Progress(progress) => {
                if lock_lifecycle(&self.state)
                    .report_progress(&job_id, progress)
                    .is_err()
                {
                    self.fail_active(
                        &job_id,
                        service_error(
                            ServiceErrorTag::WorkerProtocolFailure,
                            "Worker progress violated the lifecycle.",
                        ),
                    );
                }
            }
            AcceptedReply::Ready(ready) => {
                let result =
                    lock_lifecycle(&self.state).accept_worker_ready(&job_id, ready, Some(elapsed));
                if result.is_ok() {
                    if let Some(worker) = self.worker.as_mut() {
                        worker.active = None;
                    }
                } else {
                    self.fail_active(
                        &job_id,
                        service_error(
                            ServiceErrorTag::WorkerProtocolFailure,
                            "Worker readiness did not match the admitted host.",
                        ),
                    );
                }
            }
            AcceptedReply::Result(result) => {
                let accepted = lock_lifecycle(&self.state).accept_answer_result(
                    &job_id,
                    result,
                    Some(elapsed),
                );
                if accepted.is_ok() {
                    if let Some(worker) = self.worker.as_mut() {
                        worker.active = None;
                    }
                } else {
                    // A cancellation may have won the lifecycle lock after the
                    // reply was parsed. Preserve that winner and discard data.
                    if lock_lifecycle(&self.state)
                        .job(&job_id)
                        .is_some_and(|job| job.state == JobState::Stopping)
                    {
                        self.stop_reserved(&job_id);
                    } else {
                        self.fail_active(
                            &job_id,
                            service_error(
                                ServiceErrorTag::WorkerProtocolFailure,
                                "Worker result did not match the admitted request.",
                            ),
                        );
                    }
                }
            }
            AcceptedReply::Failure(error) => self.fail_active(&job_id, error),
            AcceptedReply::Unloaded => {
                if lock_lifecycle(&self.state)
                    .acknowledge_unloaded(&job_id)
                    .is_err()
                {
                    self.fail_active(
                        &job_id,
                        service_error(
                            ServiceErrorTag::WorkerProtocolFailure,
                            "Unload acknowledgment violated the lifecycle.",
                        ),
                    );
                    return;
                }
                if let Some(worker) = self.worker.as_mut() {
                    if let Some(exchange) = worker.active.as_mut() {
                        exchange.unload_acknowledged = true;
                    }
                    worker.awaiting_reap = true;
                }
            }
        }
    }

    fn handle_reader_closed(&mut self, generation: u64, protocol_failure: bool) {
        let Some(worker) = self
            .worker
            .as_mut()
            .filter(|worker| worker.generation == generation)
        else {
            return;
        };
        worker.awaiting_reap = true;
        let active = worker
            .active
            .as_ref()
            .map(|exchange| (exchange.job_id.clone(), exchange.unload_acknowledged));
        if let Some((job_id, unload_acknowledged)) = active {
            let stopping = lock_lifecycle(&self.state)
                .job(&job_id)
                .is_some_and(|job| job.state == JobState::Stopping);
            if protocol_failure && !stopping {
                self.fail_active(
                    &job_id,
                    service_error(
                        ServiceErrorTag::WorkerProtocolFailure,
                        "Private worker stdout ended in an invalid frame.",
                    ),
                );
                return;
            }
            if !unload_acknowledged && !stopping {
                self.fail_active(
                    &job_id,
                    service_error(
                        ServiceErrorTag::WorkerFailure,
                        "Private worker exited before its required reply.",
                    ),
                );
                return;
            }
        } else {
            self.fail_idle(service_error(
                if protocol_failure {
                    ServiceErrorTag::WorkerProtocolFailure
                } else {
                    ServiceErrorTag::WorkerFailure
                },
                if protocol_failure {
                    "Ready private worker emitted an invalid frame."
                } else {
                    "Ready private worker exited unexpectedly."
                },
            ));
            return;
        }
        self.poll_reap();
    }

    fn handle_writer_failed(&mut self, generation: u64, job_id: &str) {
        let matches = self.worker.as_ref().is_some_and(|worker| {
            worker.generation == generation
                && worker
                    .active
                    .as_ref()
                    .is_some_and(|exchange| exchange.job_id == job_id)
        });
        if matches {
            self.fail_active(
                job_id,
                service_error(
                    ServiceErrorTag::WorkerFailure,
                    "Private worker input pipe failed during a dispatched command.",
                ),
            );
        }
    }

    fn handle_stderr_read_failed(&mut self, generation: u64) {
        let active_job = self.worker.as_ref().and_then(|worker| {
            (worker.generation == generation)
                .then(|| {
                    worker
                        .active
                        .as_ref()
                        .map(|exchange| exchange.job_id.clone())
                })
                .flatten()
        });
        if let Some(job_id) = active_job {
            self.fail_active(
                &job_id,
                service_error(
                    ServiceErrorTag::WorkerFailure,
                    "Private worker stderr could not be drained.",
                ),
            );
        } else if self
            .worker
            .as_ref()
            .is_some_and(|worker| worker.generation == generation)
        {
            self.fail_idle(service_error(
                ServiceErrorTag::WorkerFailure,
                "Ready private worker stderr could not be drained.",
            ));
        }
    }

    fn handle_deadline(&mut self, generation: u64, job_id: &str) {
        let matches = self.worker.as_ref().is_some_and(|worker| {
            worker.generation == generation
                && worker
                    .active
                    .as_ref()
                    .is_some_and(|exchange| exchange.job_id == job_id)
        });
        if !matches {
            return;
        }
        let should_stop = {
            let mut state = lock_lifecycle(&self.state);
            state.job(job_id).is_some_and(|job| {
                !matches!(
                    job.state,
                    JobState::Completed | JobState::Cancelled | JobState::Failed
                )
            }) && state.deadline(job_id).is_ok()
        };
        if should_stop {
            self.stop_reserved(job_id);
        }
    }

    fn stop_reserved(&mut self, job_id: &str) {
        let Some(worker) = self.worker.as_mut() else {
            if let Some(pending) = self
                .pending_launch
                .as_mut()
                .filter(|pending| pending.job_id == job_id)
            {
                pending.deadline_armed = false;
            }
            return;
        };
        let generation = worker.generation;
        if let Some(exchange) = worker
            .active
            .as_mut()
            .filter(|exchange| exchange.job_id == job_id)
        {
            exchange.sequence.mark_parent_stop();
        } else {
            let matches_spawned_job = {
                let state = lock_lifecycle(&self.state);
                state.active_job_id() == Some(job_id)
                    && state.current_worker_generation() == Some(generation)
            };
            if !matches_spawned_job {
                return;
            }
        }
        {
            let mut state = lock_lifecycle(&self.state);
            if !state
                .job(job_id)
                .is_some_and(|job| job.state == JobState::Stopping)
            {
                return;
            }
            let _ = state.report_progress(job_id, plain_progress(ProgressStage::Terminating));
        }
        let result = launch::terminate_owned(&mut worker.child);
        let mut state = lock_lifecycle(&self.state);
        if result.is_ok() {
            let _ = state.report_progress(job_id, plain_progress(ProgressStage::Reaping));
            let _ = state.confirm_reaped(job_id, missing_unload_ack_error());
            self.worker = None;
        } else {
            let _ = state.note_termination_unconfirmed(job_id);
            worker.awaiting_reap = true;
        }
    }

    fn fail_active(&mut self, job_id: &str, cause: ServiceError) {
        let active = lock_lifecycle(&self.state).active_job_id() == Some(job_id);
        if !active {
            return;
        }
        {
            let mut state = lock_lifecycle(&self.state);
            let _ = state.worker_failure(job_id, cause);
        }
        if self.worker.is_some() {
            self.stop_reserved(job_id);
        }
    }

    fn fail_idle(&mut self, cause: ServiceError) {
        if self.worker.is_none() {
            return;
        }
        {
            let mut state = lock_lifecycle(&self.state);
            if state.idle_worker_failure(cause).is_err() {
                return;
            }
            let _ = state.report_idle_stop_progress(ProgressStage::Terminating);
        }
        let result = self
            .worker
            .as_mut()
            .map(|worker| launch::terminate_owned(&mut worker.child));
        let mut state = lock_lifecycle(&self.state);
        if result.is_some_and(|result| result.is_ok()) {
            let _ = state.report_idle_stop_progress(ProgressStage::Reaping);
            let _ = state.confirm_idle_worker_reaped();
            self.worker = None;
        } else {
            let _ = state.note_idle_termination_unconfirmed();
            if let Some(worker) = self.worker.as_mut() {
                worker.awaiting_reap = true;
            }
        }
    }

    fn poll_reap(&mut self) {
        let Some(worker) = self.worker.as_ref() else {
            return;
        };
        let active = worker.active.as_ref().map(|exchange| {
            (
                exchange.job_id.clone(),
                exchange.kind,
                exchange.unload_acknowledged,
                exchange.deadline_at,
            )
        });
        if let Some((job_id, JobKind::Unload, _, deadline_at)) = &active {
            let stopping = lock_lifecycle(&self.state)
                .job(job_id)
                .is_some_and(|job| job.state == JobState::Stopping);
            if !stopping && Instant::now() >= *deadline_at {
                let _ = lock_lifecycle(&self.state).deadline(job_id);
                self.stop_reserved(job_id);
                return;
            }
        }
        let reaped = match self.worker.as_mut() {
            Some(worker) => match worker.child.try_wait() {
                Ok(Some(_)) => true,
                Ok(None) | Err(_) => false,
            },
            None => return,
        };
        if !reaped {
            return;
        }
        let mut state = lock_lifecycle(&self.state);
        if let Some((job_id, _, _, _)) = active {
            if state
                .job(&job_id)
                .is_some_and(|job| job.state == JobState::Stopping)
            {
                let _ = state.report_progress(&job_id, plain_progress(ProgressStage::Reaping));
            }
            let _ = state.confirm_reaped(&job_id, missing_unload_ack_error());
        } else {
            let _ = state.report_idle_stop_progress(ProgressStage::Reaping);
            let _ = state.confirm_idle_worker_reaped();
        }
        drop(state);
        self.worker = None;
    }

    fn poll_unadopted_child(&mut self) {
        let reaped = match self.unadopted_child.as_mut() {
            Some(worker) => match worker.child.try_wait() {
                Ok(Some(_)) => true,
                Ok(None) | Err(_) => false,
            },
            None => return,
        };
        if !reaped {
            return;
        }
        let Some(worker) = self.unadopted_child.take() else {
            return;
        };
        let _ = lock_lifecycle(&self.state)
            .finish_load_launch_without_child(&worker.job_id, worker.cause.clone());
    }

    fn shutdown(&mut self) {
        if let Some(pending) = self.pending_launch.take() {
            if let Ok(Ok(mut child)) = pending.thread.join() {
                force_reap_for_cleanup(&mut child);
            }
        }
        drop(self.unadopted_child.take());
        let Some(mut worker) = self.worker.take() else {
            return;
        };
        if let Some(exchange) = worker.active.as_mut() {
            exchange.sequence.mark_parent_stop();
        }
        drop(worker);
    }
}

fn force_reap_for_cleanup(child: &mut Child) {
    if launch::terminate_owned(child).is_ok() {
        return;
    }
    // Drop/unwind must retain the exact owned handle until the child is known
    // gone. Shutdown latency is intentionally unbounded in this fail-closed
    // path; the public work deadlines remain separate control policies.
    loop {
        let _ = child.kill();
        match child.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) | Err(_) => thread::sleep(REAP_POLL),
        }
    }
}

fn start_stdin_writer(
    mut stdin: ChildStdin,
    worker_generation: u64,
    events: mpsc::SyncSender<ControllerEvent>,
) -> Option<(mpsc::SyncSender<IpcRequest>, JoinHandle<()>)> {
    let (sender, receiver) = mpsc::sync_channel::<IpcRequest>(1);
    let thread = thread::Builder::new()
        .name("r4-workbench-worker-writer".to_owned())
        .spawn(move || {
            while let Ok(request) = receiver.recv() {
                let job_id = request.job_id.clone();
                if ipc::write_request(&mut stdin, &request).is_err() {
                    let _ = events.try_send(ControllerEvent::WriterFailed {
                        worker_generation,
                        job_id,
                    });
                    break;
                }
            }
        })
        .ok()?;
    Some((sender, thread))
}

fn start_stdout_reader(
    mut stdout: ChildStdout,
    worker_generation: u64,
    events: mpsc::SyncSender<ControllerEvent>,
) -> bool {
    thread::Builder::new()
        .name("r4-workbench-worker-replies".to_owned())
        .spawn(move || loop {
            match ipc::read_response_or_eof(&mut stdout) {
                Ok(Some(response)) => {
                    if events
                        .send(ControllerEvent::Reply {
                            worker_generation,
                            response,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
                Ok(None) => {
                    let _ = events.send(ControllerEvent::ReaderClosed {
                        worker_generation,
                        protocol_failure: false,
                    });
                    break;
                }
                Err(error) => {
                    let _ = events.send(ControllerEvent::ReaderClosed {
                        worker_generation,
                        protocol_failure: error.service_tag()
                            == ServiceErrorTag::WorkerProtocolFailure,
                    });
                    break;
                }
            }
        })
        .is_ok()
}

fn start_stderr_reader(
    mut stderr: ChildStderr,
    worker_generation: u64,
    events: mpsc::SyncSender<ControllerEvent>,
) -> bool {
    thread::Builder::new()
        .name("r4-workbench-worker-stderr".to_owned())
        .spawn(move || {
            let mut chunk = [0_u8; 4_096];
            let mut retained = Vec::with_capacity(STDERR_RETAIN_BYTES);
            let mut truncated = false;
            let mut read_failed = false;
            loop {
                match stderr.read(&mut chunk) {
                    Ok(0) => break,
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                    Err(_) => {
                        read_failed = true;
                        break;
                    }
                    Ok(count) => {
                        retain_stderr_chunk(&mut retained, &mut truncated, &chunk[..count])
                    }
                }
            }
            let _ = events.send(ControllerEvent::Stderr {
                worker_generation,
                bytes: retained,
                truncated,
            });
            if read_failed {
                let _ = events.send(ControllerEvent::StderrReadFailed { worker_generation });
            }
        })
        .is_ok()
}

fn retain_stderr_chunk(retained: &mut Vec<u8>, truncated: &mut bool, chunk: &[u8]) {
    if *truncated || chunk.is_empty() {
        return;
    }
    let content_limit = STDERR_RETAIN_BYTES.saturating_sub(STDERR_TRUNCATION.len());
    let remaining = content_limit.saturating_sub(retained.len());
    let take = remaining.min(chunk.len());
    retained.extend(sanitize_stderr(&chunk[..take]));
    if take != chunk.len() {
        retained.extend_from_slice(STDERR_TRUNCATION);
        retained.truncate(STDERR_RETAIN_BYTES);
        *truncated = true;
    }
}

fn sanitize_stderr(bytes: &[u8]) -> Vec<u8> {
    bytes
        .iter()
        .map(|byte| match byte {
            b'\n' | b'\r' | b'\t' => *byte,
            0x20..=0x7e => *byte,
            _ => b'?',
        })
        .collect()
}

fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis())
        .unwrap_or(crate::wire::UINT53_MAX)
        .min(crate::wire::UINT53_MAX)
}

fn plain_progress(stage: ProgressStage) -> crate::wire::Progress {
    crate::wire::Progress {
        stage,
        completed: None,
        total: None,
        unit: None,
        fraction: None,
        eta_ms: None,
    }
}

fn missing_unload_ack_error() -> ServiceError {
    service_error(
        ServiceErrorTag::WorkerFailure,
        "Private worker exited without the required unload acknowledgment.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_mapping_matches_the_frozen_error_table() {
        assert_eq!(status_for_tag(ServiceErrorTag::InvalidBase64), 400);
        assert_eq!(status_for_tag(ServiceErrorTag::StaleInstance), 409);
        assert_eq!(status_for_tag(ServiceErrorTag::RawInputTooLarge), 413);
        assert_eq!(
            status_for_tag(ServiceErrorTag::UnavailableNativeQualification),
            503
        );
        assert_eq!(status_for_tag(ServiceErrorTag::DeadlineExceeded), 504);
    }

    #[test]
    fn host_and_startup_errors_are_bounded_typed_utf8() {
        let error = service_error(
            ServiceErrorTag::WorkerFailure,
            format!("\0{}tail", "é".repeat(300)),
        );
        assert_eq!(error.message.len(), 512);
        assert!(!error.message.chars().any(char::is_control));
        error.validate().expect("bounded host error");

        let startup = StartupError::bad_request("Synthetic root configuration was rejected.");
        startup.error.validate().expect("typed startup error");
        assert!(startup.to_string().contains("BAD_REQUEST"));
        assert!(startup.to_string().len() < 1_024);
    }

    #[test]
    fn request_extent_is_bounded_and_waits_for_the_exact_body() {
        let header = b"POST /uor/v1/workbench/requests HTTP/1.1\r\nContent-Length: 3\r\n\r\n";
        assert!(!request_is_complete(header));
        let mut partial = header.to_vec();
        partial.extend_from_slice(b"ab");
        assert!(!request_is_complete(&partial));
        partial.push(b'c');
        assert!(request_is_complete(&partial));
        assert!(request_is_complete(&vec![b'x'; HEADER_MAX_BYTES + 1]));
    }

    #[test]
    fn stderr_retention_sanitizes_and_never_exceeds_the_cap() {
        assert_eq!(sanitize_stderr(&[b'a', 0, 0xff, b'\n']), b"a??\n");
        let mut retained = Vec::new();
        let mut truncated = false;
        retain_stderr_chunk(
            &mut retained,
            &mut truncated,
            &vec![b'a'; STDERR_RETAIN_BYTES + 10],
        );
        assert_eq!(retained.len(), STDERR_RETAIN_BYTES);
        assert!(truncated);
        assert!(retained.ends_with(STDERR_TRUNCATION));
        let frozen = retained.clone();
        retain_stderr_chunk(&mut retained, &mut truncated, b"discarded");
        assert_eq!(retained, frozen);
    }
}
