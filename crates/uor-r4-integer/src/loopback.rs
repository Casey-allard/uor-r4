use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::openai::{parse_json_value, GenerationFamily, OpenAiError, OpenAiServing, Utf8Buffer};
use crate::{IntegerError, Result as IntegerResult};
use uor_r4_vision::{decompose_rgba, Grid, VisionError, MAX_GRID};

const HEADER_LIMIT: usize = 32_768;

#[derive(Clone)]
pub struct LoopbackConfig {
    pub serving: Arc<OpenAiServing>,
    pub host: String,
    pub port: u16,
    pub token: String,
    pub origin: Option<String>,
    pub studio_root: Option<PathBuf>,
    pub max_body_bytes: usize,
}

impl LoopbackConfig {
    pub fn new(serving: OpenAiServing) -> IntegerResult<Self> {
        Ok(Self {
            serving: Arc::new(serving),
            host: "127.0.0.1".into(),
            port: 8000,
            token: fresh_launch_token()?,
            origin: None,
            studio_root: None,
            max_body_bytes: 1_048_576,
        })
    }
}

pub struct LoopbackServer {
    port: u16,
    token: String,
    shutdown: Arc<AtomicBool>,
    join: Option<JoinHandle<IntegerResult<()>>>,
}

impl LoopbackServer {
    pub fn spawn(mut config: LoopbackConfig) -> IntegerResult<Self> {
        validate_config(&config)?;
        let listener = TcpListener::bind((config.host.as_str(), config.port))?;
        let port = listener.local_addr()?.port();
        config.port = port;
        let token = config.token.clone();
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker_shutdown = Arc::clone(&shutdown);
        let join = std::thread::Builder::new()
            .name("uor-openai-loopback".into())
            .spawn(move || serve_on(listener, config, worker_shutdown))?;
        Ok(Self {
            port,
            token,
            shutdown,
            join: Some(join),
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn shutdown(mut self) -> IntegerResult<()> {
        self.shutdown.store(true, Ordering::Release);
        let _ = TcpStream::connect(("127.0.0.1", self.port));
        match self.join.take() {
            Some(join) => match join.join() {
                Ok(result) => result,
                Err(_) => Err(IntegerError::Invalid("loopback worker panicked".into())),
            },
            None => Ok(()),
        }
    }
}

impl Drop for LoopbackServer {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        let _ = TcpStream::connect(("127.0.0.1", self.port));
    }
}

pub fn serve_loopback(mut config: LoopbackConfig) -> IntegerResult<()> {
    validate_config(&config)?;
    let listener = TcpListener::bind((config.host.as_str(), config.port))?;
    config.port = listener.local_addr()?.port();
    println!(
        "uor-chat OpenAI loopback listening on http://127.0.0.1:{} (token required for /v1)",
        config.port
    );
    serve_on(listener, config, Arc::new(AtomicBool::new(false)))
}

fn validate_config(config: &LoopbackConfig) -> IntegerResult<()> {
    if config.host != "127.0.0.1" {
        return Err(IntegerError::Invalid(
            "OpenAI serving binds only 127.0.0.1".into(),
        ));
    }
    if config.token.is_empty() {
        return Err(IntegerError::Invalid(
            "a nonempty per-launch token is required".into(),
        ));
    }
    if config.origin.as_ref().is_some_and(|origin| {
        origin.is_empty()
            || origin.bytes().any(|byte| byte <= 0x20 || byte == 0x7f)
            || !(origin.starts_with("http://") || origin.starts_with("https://"))
    }) {
        return Err(IntegerError::Invalid(
            "origin must be one nonempty HTTP(S) origin without control characters".into(),
        ));
    }
    if config.max_body_bytes == 0 {
        return Err(IntegerError::Invalid(
            "max_body_bytes must be positive".into(),
        ));
    }
    Ok(())
}

fn serve_on(
    listener: TcpListener,
    config: LoopbackConfig,
    shutdown: Arc<AtomicBool>,
) -> IntegerResult<()> {
    loop {
        if shutdown.load(Ordering::Acquire) {
            return Ok(());
        }
        let (stream, _) = match listener.accept() {
            Ok(connection) => connection,
            Err(_) if shutdown.load(Ordering::Acquire) => return Ok(()),
            Err(error) => return Err(IntegerError::Io(error)),
        };
        let config = config.clone();
        std::thread::Builder::new()
            .name("uor-openai-request".into())
            .spawn(move || {
                let _ = handle_connection(stream, &config);
            })?;
    }
}

struct Request {
    method: String,
    target: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Request {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

fn handle_connection(mut stream: TcpStream, config: &LoopbackConfig) -> Result<(), OpenAiError> {
    stream.set_read_timeout(Some(Duration::from_secs(30))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(30))).ok();
    let mut response_started = false;
    let outcome = (|| {
        let request = read_request(&mut stream, config.max_body_bytes)?;
        validate_host_origin(&request, config)?;
        if request.target == "/healthz" && request.method == "GET" {
            return write_json(
                &mut stream,
                200,
                &config.serving.health_json(),
                config,
                &mut response_started,
            );
        }
        if request.target.starts_with("/v1/") {
            if request.method == "OPTIONS" {
                return route_api(&mut stream, request, config, &mut response_started);
            }
            authenticate(&request, &config.token)?;
            return route_api(&mut stream, request, config, &mut response_started);
        }
        if request.method == "GET" && config.studio_root.is_some() {
            return serve_static(&mut stream, &request, config, &mut response_started);
        }
        Err(OpenAiError::unsupported("route is not implemented").with_code("not_implemented"))
    })();
    if let Err(error) = outcome {
        write_connection_error(&mut stream, &error, config, response_started)?;
    }
    Ok(())
}

pub(crate) fn write_connection_error(
    stream: &mut TcpStream,
    error: &OpenAiError,
    config: &LoopbackConfig,
    response_started: bool,
) -> Result<(), OpenAiError> {
    if response_started {
        return Ok(());
    }
    let mut error_response_started = false;
    write_json(
        stream,
        error.status,
        &error.to_json(),
        config,
        &mut error_response_started,
    )
}

fn read_request(stream: &mut TcpStream, max_body: usize) -> Result<Request, OpenAiError> {
    let mut reader = BufReader::new(stream);
    let mut lines = Vec::new();
    let mut consumed = 0usize;
    loop {
        let line = read_header_line(&mut reader, &mut consumed)?;
        if line == "\r\n" || line == "\n" {
            break;
        }
        lines.push(line);
    }
    let request_line = lines
        .first()
        .ok_or_else(|| OpenAiError::new(400, "missing HTTP request line"))?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_owned();
    let target = parts.next().unwrap_or_default().to_owned();
    let version = parts.next().unwrap_or_default();
    if parts.next().is_some()
        || !matches!(method.as_str(), "GET" | "POST" | "OPTIONS")
        || version != "HTTP/1.1"
        || target.is_empty()
    {
        return Err(OpenAiError::new(400, "unsupported HTTP request line"));
    }
    let mut headers = Vec::new();
    for line in &lines[1..] {
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| OpenAiError::new(400, "malformed HTTP header"))?;
        if name.is_empty()
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return Err(OpenAiError::new(400, "invalid HTTP header name"));
        }
        headers.push((name.to_owned(), value.trim().to_owned()));
    }
    for singular in [
        "host",
        "content-length",
        "content-type",
        "authorization",
        "x-uor-studio-token",
        "origin",
    ] {
        if headers
            .iter()
            .filter(|(name, _)| name.eq_ignore_ascii_case(singular))
            .count()
            > 1
        {
            return Err(OpenAiError::new(
                400,
                format!("multiple {singular} headers are refused"),
            ));
        }
    }
    let request = Request {
        method,
        target,
        headers,
        body: Vec::new(),
    };
    if request.header("transfer-encoding").is_some() {
        return Err(OpenAiError::new(400, "transfer encoding is not supported"));
    }
    let length = match request.header("content-length") {
        Some(value) => value
            .parse::<usize>()
            .map_err(|_| OpenAiError::new(400, "invalid Content-Length"))?,
        None if request.method == "POST" => {
            return Err(OpenAiError::new(400, "POST requires Content-Length"))
        }
        None => 0,
    };
    if length > max_body {
        let mut remaining = length.min(max_body.saturating_add(1));
        let deadline = std::time::Instant::now() + Duration::from_millis(250);
        let mut buffer = [0u8; 2048];
        while remaining > 0 {
            let now = std::time::Instant::now();
            if now >= deadline {
                break;
            }
            let _ = reader
                .get_ref()
                .set_read_timeout(Some(deadline.saturating_duration_since(now)));
            let chunk = remaining.min(buffer.len());
            match reader.read(&mut buffer[..chunk]) {
                Ok(0) | Err(_) => break,
                Ok(read) => remaining -= read,
            }
        }
        return Err(OpenAiError::new(
            413,
            "HTTP body exceeds the configured limit",
        ));
    }
    if request.method != "POST" && length != 0 {
        return Err(OpenAiError::new(400, "this method must not carry a body"));
    }
    let mut body = vec![0u8; length];
    reader
        .read_exact(&mut body)
        .map_err(|_| OpenAiError::new(400, "HTTP body is shorter than Content-Length"))?;
    Ok(Request { body, ..request })
}

fn read_header_line<R: BufRead>(
    reader: &mut R,
    consumed: &mut usize,
) -> Result<String, OpenAiError> {
    let mut line = Vec::new();
    loop {
        let available = reader
            .fill_buf()
            .map_err(|_| OpenAiError::new(400, "could not read HTTP headers"))?;
        if available.is_empty() {
            return Err(OpenAiError::new(400, "incomplete HTTP headers"));
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let take = newline.map_or(available.len(), |index| index + 1);
        if line.len() + take > 4096 || *consumed + take > HEADER_LIMIT {
            return Err(OpenAiError::new(
                400,
                "invalid or oversized HTTP header line",
            ));
        }
        line.extend_from_slice(&available[..take]);
        *consumed += take;
        reader.consume(take);
        if newline.is_some() {
            return String::from_utf8(line)
                .map_err(|_| OpenAiError::new(400, "HTTP headers must be valid UTF-8"));
        }
    }
}

fn validate_host_origin(request: &Request, config: &LoopbackConfig) -> Result<(), OpenAiError> {
    let expected = format!("127.0.0.1:{}", config.port);
    if request.header("host") != Some(expected.as_str()) {
        return Err(
            OpenAiError::new(400, "Host does not match the loopback authority")
                .with_code("host_rejected"),
        );
    }
    if let Some(origin) = request.header("origin") {
        let same_origin = format!("http://127.0.0.1:{}", config.port);
        if origin != same_origin && config.origin.as_deref() != Some(origin) {
            return Err(OpenAiError::new(403, "Origin is not allowed").with_code("origin_rejected"));
        }
    }
    Ok(())
}

fn authenticate(request: &Request, token: &str) -> Result<(), OpenAiError> {
    let expected = format!("Bearer {token}");
    if request
        .header("authorization")
        .is_some_and(|value| !constant_time_eq(value.as_bytes(), expected.as_bytes()))
    {
        return Err(OpenAiError::new(401, "invalid bearer token").with_code("unauthorized"));
    }
    if request
        .header("x-uor-studio-token")
        .is_some_and(|value| !constant_time_eq(value.as_bytes(), token.as_bytes()))
    {
        return Err(OpenAiError::new(401, "invalid Studio token").with_code("unauthorized"));
    }
    if request.header("authorization").is_none() && request.header("x-uor-studio-token").is_none() {
        return Err(
            OpenAiError::new(401, "a bearer or X-UOR-Studio-Token token is required")
                .with_code("unauthorized"),
        );
    }
    Ok(())
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let mut difference = left.len() ^ right.len();
    for (index, byte) in right.iter().enumerate() {
        difference |= usize::from(left.get(index).copied().unwrap_or(0) ^ byte);
    }
    for (index, byte) in left.iter().enumerate() {
        difference |= usize::from(right.get(index).copied().unwrap_or(0) ^ byte);
    }
    difference == 0
}

fn route_api(
    stream: &mut TcpStream,
    request: Request,
    config: &LoopbackConfig,
    response_started: &mut bool,
) -> Result<(), OpenAiError> {
    if request.method == "OPTIONS" {
        return write_response(
            stream,
            204,
            "application/json; charset=utf-8",
            &[],
            config,
            response_started,
        );
    }
    let (path, query) = request
        .target
        .split_once('?')
        .map_or((request.target.as_str(), None), |(path, query)| {
            (path, Some(query))
        });
    match (request.method.as_str(), path) {
        ("GET", "/v1/models") => write_json(
            stream,
            200,
            &config.serving.models_json(),
            config,
            response_started,
        ),
        ("GET", "/v1/capabilities") => write_json(
            stream,
            200,
            &config.serving.capabilities_json(),
            config,
            response_started,
        ),
        ("GET", path) if path.starts_with("/v1/models/") => {
            if path == "/v1/models/uor-chat" {
                write_json(
                    stream,
                    200,
                    &config.serving.model_card(),
                    config,
                    response_started,
                )
            } else {
                Err(OpenAiError::new(404, "model does not exist").with_code("model_not_found"))
            }
        }
        ("POST", "/v1/chat/completions") => generate_api(
            stream,
            &request,
            config,
            GenerationFamily::Chat,
            response_started,
        ),
        ("POST", "/v1/completions") => generate_api(
            stream,
            &request,
            config,
            GenerationFamily::Completion,
            response_started,
        ),
        ("POST", "/v1/responses") => generate_api(
            stream,
            &request,
            config,
            GenerationFamily::Responses,
            response_started,
        ),
        ("POST", "/v1/images/decompose") => decompose_image_api(
            stream,
            &request,
            query.unwrap_or_default(),
            config,
            response_started,
        ),
        _ => Err(
            OpenAiError::unsupported("this OpenAI endpoint family is not implemented")
                .with_code("not_implemented"),
        ),
    }
}

fn decompose_image_api(
    stream: &mut TcpStream,
    request: &Request,
    query: &str,
    config: &LoopbackConfig,
    response_started: &mut bool,
) -> Result<(), OpenAiError> {
    if !request
        .header("content-type")
        .is_some_and(|value| value == "application/octet-stream")
    {
        return Err(OpenAiError::new(
            400,
            "Content-Type must be application/octet-stream",
        ));
    }
    let (width, height, grid) = parse_image_query(query)?;
    let image = decompose_rgba(width, height, &request.body, grid).map_err(image_error)?;
    let decomposition_id = config.serving.issue_decomposition(image.clone())?;
    let canonical = image.canonical_bytes();
    let headers = format!(
        "X-UOR-Image-Schema: uor-r4.image-decomposition/1\r\nX-UOR-Source-Blake3: {}\r\nX-UOR-Image-Width: {}\r\nX-UOR-Image-Height: {}\r\nX-UOR-Grid-Columns: {}\r\nX-UOR-Grid-Rows: {}\r\nX-UOR-Patch-Count: {}\r\nX-UOR-Decomposition-Id: {}\r\n",
        image.source_hash_hex(),
        image.width(),
        image.height(),
        image.grid().columns(),
        image.grid().rows(),
        image.patches().len(),
        decomposition_id
    );
    write_response_with_headers(
        stream,
        200,
        "application/octet-stream",
        &canonical,
        config,
        response_started,
        &headers,
    )
}

fn parse_image_query(query: &str) -> Result<(u32, u32, Grid), OpenAiError> {
    let mut width = None;
    let mut height = None;
    let mut columns = None;
    let mut rows = None;
    if query.is_empty() {
        return Err(OpenAiError::new(
            400,
            "width, height, columns and rows query parameters are required",
        ));
    }
    for pair in query.split('&') {
        let (name, value) = pair
            .split_once('=')
            .ok_or_else(|| OpenAiError::new(400, "image query parameters must use name=value"))?;
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(OpenAiError::new(
                400,
                format!("image query parameter {name:?} must be a decimal integer"),
            ));
        }
        let parsed = value.parse::<u32>().map_err(|_| {
            OpenAiError::new(400, format!("image query parameter {name:?} is too large"))
        })?;
        let slot = match name {
            "width" => &mut width,
            "height" => &mut height,
            "columns" => &mut columns,
            "rows" => &mut rows,
            _ => {
                return Err(OpenAiError::new(
                    400,
                    format!("unsupported image query parameter {name:?}"),
                ))
            }
        };
        if slot.replace(parsed).is_some() {
            return Err(OpenAiError::new(
                400,
                format!("image query parameter {name:?} is repeated"),
            ));
        }
    }
    let width = width.ok_or_else(|| OpenAiError::new(400, "width is required"))?;
    let height = height.ok_or_else(|| OpenAiError::new(400, "height is required"))?;
    let columns = bounded_grid_axis("columns", columns)?;
    let rows = bounded_grid_axis("rows", rows)?;
    Ok((width, height, Grid::new(columns, rows)))
}

fn bounded_grid_axis(field: &str, value: Option<u32>) -> Result<u16, OpenAiError> {
    let value = value.ok_or_else(|| OpenAiError::new(400, format!("{field} is required")))?;
    if value > u32::from(MAX_GRID) {
        return Err(OpenAiError::new(
            413,
            format!("{field} exceeds the {MAX_GRID} image grid limit"),
        )
        .with_code("image_limit"));
    }
    u16::try_from(value).map_err(|_| OpenAiError::new(400, format!("{field} exceeds u16")))
}

fn image_error(error: VisionError) -> OpenAiError {
    match error {
        VisionError::DimensionOutOfRange { value: 0, .. } => {
            OpenAiError::new(400, error.to_string())
        }
        VisionError::DimensionOutOfRange { .. }
        | VisionError::GridOutOfRange { .. }
        | VisionError::ArithmeticOverflow => {
            OpenAiError::new(413, error.to_string()).with_code("image_limit")
        }
        VisionError::PixelBufferLength { .. }
        | VisionError::EmptyGrid
        | VisionError::GridLargerThanImage { .. }
        | VisionError::InvalidCanonical(_) => OpenAiError::new(400, error.to_string()),
    }
}

fn generate_api(
    stream: &mut TcpStream,
    request: &Request,
    config: &LoopbackConfig,
    family: GenerationFamily,
    response_started: &mut bool,
) -> Result<(), OpenAiError> {
    if !request.header("content-type").is_some_and(|value| {
        value == "application/json" || value == "application/json; charset=utf-8"
    }) {
        return Err(OpenAiError::new(
            400,
            "Content-Type must be application/json",
        ));
    }
    let input = parse_json_value(&request.body)?;
    let generation = config.serving.validate_request(family, &input)?;
    if generation.stream {
        stream_generation(stream, &generation, config, family, response_started)
    } else {
        let messages = generation.borrowed_messages();
        let outcome = config
            .serving
            .generate(&messages, generation.max_tokens, |_, _| Ok(()))?;
        write_json(
            stream,
            200,
            &config.serving.response_json(family, &outcome),
            config,
            response_started,
        )
    }
}

fn stream_generation(
    stream: &mut TcpStream,
    generation: &crate::openai::GenerationRequest,
    config: &LoopbackConfig,
    family: GenerationFamily,
    response_started: &mut bool,
) -> Result<(), OpenAiError> {
    let _permit = config.serving.try_start_generation()?;
    let identity = config.serving.stream_identity(family);
    *response_started = true;
    write_sse_headers(stream, config)?;
    write_sse_event(
        stream,
        family,
        &config.serving.stream_role_chunk(family, &identity),
    )?;
    let mut utf8 = Utf8Buffer::default();
    let protocol = config.serving.protocol()?;
    let eos = protocol.eos_id;
    let mut decoded = String::new();
    let mut emitted_bytes = 0;
    let outcome = config.serving.generate_with_permit(
        &generation.borrowed_messages(),
        generation.max_tokens,
        |id, bytes| {
            let text = if id == eos {
                String::new()
            } else {
                decoded.push_str(&utf8.push(bytes)?);
                let visible = protocol.reply_text(&decoded);
                let text = visible.get(emitted_bytes..).ok_or_else(|| {
                    OpenAiError::new(500, "streaming reply text is not append-only")
                })?;
                emitted_bytes = visible.len();
                text.to_owned()
            };
            write_sse_event(
                stream,
                family,
                &config
                    .serving
                    .stream_token_chunk(family, &identity, id, &text),
            )
        },
    )?;
    utf8.finish()?;
    if protocol.reply_text(&decoded) != outcome.reply.text {
        return Err(OpenAiError::new(
            500,
            "streaming reply text diverged from the batch reply",
        ));
    }
    write_sse_event(
        stream,
        family,
        &config
            .serving
            .stream_finish_chunk(family, &identity, &outcome),
    )?;
    if generation.include_usage && family != GenerationFamily::Responses {
        write_sse_event(
            stream,
            family,
            &config
                .serving
                .stream_usage_chunk(family, &identity, &outcome),
        )?;
    }
    stream
        .write_all(b"data: [DONE]\r\n\r\n")
        .map_err(|_| OpenAiError::new(500, "client disconnected during stream"))?;
    stream
        .flush()
        .map_err(|_| OpenAiError::new(500, "client disconnected during stream"))
}

fn write_sse_headers(stream: &mut TcpStream, config: &LoopbackConfig) -> Result<(), OpenAiError> {
    let headers = format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream; charset=utf-8\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n{}\r\n", cors_headers(config));
    stream
        .write_all(headers.as_bytes())
        .map_err(|_| OpenAiError::new(500, "client disconnected before stream"))
}

fn write_sse_event(
    stream: &mut TcpStream,
    family: GenerationFamily,
    value: &Value,
) -> Result<(), OpenAiError> {
    if family == GenerationFamily::Responses {
        if let Some(event) = value.get("type").and_then(Value::as_str) {
            stream
                .write_all(format!("event: {event}\r\n").as_bytes())
                .map_err(|_| OpenAiError::new(500, "client disconnected during stream"))?;
        }
    }
    let bytes =
        serde_json::to_vec(value).map_err(|error| OpenAiError::new(500, error.to_string()))?;
    stream
        .write_all(b"data: ")
        .map_err(|_| OpenAiError::new(500, "client disconnected during stream"))?;
    stream
        .write_all(&bytes)
        .map_err(|_| OpenAiError::new(500, "client disconnected during stream"))?;
    stream
        .write_all(b"\r\n\r\n")
        .map_err(|_| OpenAiError::new(500, "client disconnected during stream"))?;
    stream
        .flush()
        .map_err(|_| OpenAiError::new(500, "client disconnected during stream"))
}

fn write_json(
    stream: &mut TcpStream,
    status: u16,
    value: &Value,
    config: &LoopbackConfig,
    response_started: &mut bool,
) -> Result<(), OpenAiError> {
    let body =
        serde_json::to_vec(value).map_err(|error| OpenAiError::new(500, error.to_string()))?;
    write_response(
        stream,
        status,
        "application/json; charset=utf-8",
        &body,
        config,
        response_started,
    )
}

pub(crate) fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
    config: &LoopbackConfig,
    response_started: &mut bool,
) -> Result<(), OpenAiError> {
    write_response_with_headers(
        stream,
        status,
        content_type,
        body,
        config,
        response_started,
        "",
    )
}

fn write_response_with_headers(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
    config: &LoopbackConfig,
    response_started: &mut bool,
    extra_headers: &str,
) -> Result<(), OpenAiError> {
    let reason = match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        413 => "Payload Too Large",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        _ => "Error",
    };
    let headers = format!("HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n{extra_headers}{}\r\n", body.len(), cors_headers(config));
    *response_started = true;
    stream
        .write_all(headers.as_bytes())
        .map_err(|_| OpenAiError::new(500, "client disconnected"))?;
    stream
        .write_all(body)
        .map_err(|_| OpenAiError::new(500, "client disconnected"))?;
    stream
        .flush()
        .map_err(|_| OpenAiError::new(500, "client disconnected"))
}

fn cors_headers(config: &LoopbackConfig) -> String {
    config.origin.as_ref().map(|origin| format!("Access-Control-Allow-Origin: {origin}\r\nAccess-Control-Allow-Headers: Authorization, Content-Type, X-UOR-Studio-Token\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Expose-Headers: X-UOR-Image-Schema, X-UOR-Source-Blake3, X-UOR-Image-Width, X-UOR-Image-Height, X-UOR-Grid-Columns, X-UOR-Grid-Rows, X-UOR-Patch-Count, X-UOR-Decomposition-Id\r\n")).unwrap_or_default()
}

fn serve_static(
    stream: &mut TcpStream,
    request: &Request,
    config: &LoopbackConfig,
    response_started: &mut bool,
) -> Result<(), OpenAiError> {
    let root = config
        .studio_root
        .as_deref()
        .ok_or_else(|| OpenAiError::new(404, "Studio is not configured"))?;
    let root = root
        .canonicalize()
        .map_err(|_| OpenAiError::new(404, "Studio root is unavailable"))?;
    let target = request.target.split('?').next().unwrap_or("/");
    let relative = percent_decode(target)?;
    let relative = relative.trim_start_matches('/');
    let relative = if relative.is_empty() {
        "index.html"
    } else {
        relative
    };
    let path = Path::new(relative);
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(OpenAiError::new(404, "static path is refused"));
    }
    let candidate = root.join(path);
    for ancestor in candidate.ancestors() {
        if !ancestor.starts_with(&root) {
            break;
        }
        if std::fs::symlink_metadata(ancestor)
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false)
        {
            return Err(OpenAiError::new(404, "static symlinks are refused"));
        }
    }
    let candidate = candidate
        .canonicalize()
        .map_err(|_| OpenAiError::new(404, "static file is unavailable"))?;
    if !candidate.starts_with(&root) || !candidate.is_file() {
        return Err(OpenAiError::new(404, "static file is unavailable"));
    }
    let mime = match candidate
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
    {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "wasm" => "application/wasm",
        "lut" | "bin" => "application/octet-stream",
        _ => "application/octet-stream",
    };
    let body = if candidate == root.join("index.html") {
        let html = std::fs::read_to_string(&candidate)
            .map_err(|_| OpenAiError::new(404, "Studio index is unavailable"))?;
        let bootstrap = serde_json::json!({"baseUrl": "/v1", "model": config.serving.model_id(), "token": config.token, "protocol": "openai", "temperature": 0, "ephemeral": true})
            .to_string()
            .replace('<', "\\u003c");
        let script = format!(
            "<script>window.__UOR_NATIVE_BOOTSTRAP__=Object.freeze({});</script>",
            bootstrap
        );
        html.replace("</head>", &format!("{script}</head>"))
            .into_bytes()
    } else {
        std::fs::read(&candidate)
            .map_err(|_| OpenAiError::new(404, "static file is unavailable"))?
    };
    write_response(stream, 200, mime, &body, config, response_started)
}

fn percent_decode(target: &str) -> Result<String, OpenAiError> {
    let bytes = target.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err(OpenAiError::new(400, "invalid static-path escape"));
            }
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3])
                .map_err(|_| OpenAiError::new(400, "invalid static-path escape"))?;
            let value = u8::from_str_radix(hex, 16)
                .map_err(|_| OpenAiError::new(400, "invalid static-path escape"))?;
            decoded.push(value);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    if decoded.contains(&0) || decoded.contains(&b'\\') {
        return Err(OpenAiError::new(400, "static path is refused"));
    }
    String::from_utf8(decoded).map_err(|_| OpenAiError::new(400, "static path is not UTF-8"))
}

fn fresh_launch_token() -> IntegerResult<String> {
    let mut bytes = [0u8; 32];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut bytes))
        .map_err(IntegerError::Io)?;
    Ok(hex::encode(bytes))
}
