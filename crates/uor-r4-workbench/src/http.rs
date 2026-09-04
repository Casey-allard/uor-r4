//! Strict, single-request HTTP/1.1 admission for the loopback workbench.
//!
//! This module is deliberately transport-only. It parses an already bounded
//! byte buffer, classifies the seven public routes (or an exactly verified
//! static asset), and serializes complete close-after-response messages. It
//! neither binds a socket nor performs lifecycle or model work.

use crate::wire::{is_job_id, ServiceError, ServiceErrorTag};
use std::fmt;

pub const HEADER_MAX_BYTES: usize = 8_192;
pub const BODY_MAX_BYTES: usize = 16_384;
pub const REQUEST_READ_DEADLINE_MS: u64 = 5_000;
pub const MAX_CONNECTIONS: usize = 16;
pub const API_NAMESPACE: &str = "/uor/v1/workbench";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    Capabilities,
    Model,
    Load,
    Unload,
    Requests,
    Job {
        job_id: String,
    },
    Cancel {
        job_id: String,
    },
    /// A manifest-relative path whose exact membership was checked by the
    /// caller-provided verified asset set. `/` is represented as `index.html`.
    Static {
        manifest_path: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub route: Route,
    /// Exact bytes following the header terminator. No decoding, trimming, or
    /// normalization occurs at this layer.
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpAdmissionError {
    pub tag: ServiceErrorTag,
    pub status: u16,
    pub message: &'static str,
    /// Value for an `Allow` response header when the target is known but the
    /// method is not. This is never derived from request bytes.
    pub allow: Option<&'static str>,
}

impl HttpAdmissionError {
    fn new(tag: ServiceErrorTag, status: u16, message: &'static str) -> Self {
        Self {
            tag,
            status,
            message,
            allow: None,
        }
    }

    fn bad_request(message: &'static str) -> Self {
        Self::new(ServiceErrorTag::BadRequest, 400, message)
    }

    fn method_not_allowed(allow: &'static str) -> Self {
        Self {
            tag: ServiceErrorTag::MethodNotAllowed,
            status: 405,
            message: "method is not allowed for this route",
            allow: Some(allow),
        }
    }

    fn not_found() -> Self {
        Self::new(ServiceErrorTag::NotFound, 404, "route was not found")
    }

    pub fn as_service_error(&self) -> ServiceError {
        ServiceError {
            tag: self.tag,
            message: self.message.to_owned(),
            native: None,
        }
    }
}

impl fmt::Display for HttpAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message)
    }
}

impl std::error::Error for HttpAdmissionError {}

#[derive(Default)]
struct Headers<'a> {
    host: Option<&'a str>,
    origin: Option<&'a str>,
    content_length: Option<usize>,
    content_type: Option<&'a str>,
}

enum TargetKind {
    Capabilities,
    Model,
    Load,
    Unload,
    Requests,
    Job(String),
    Cancel(String),
    Static(String),
}

impl TargetKind {
    fn allowed_method(&self) -> HttpMethod {
        match self {
            Self::Capabilities | Self::Model | Self::Job(_) | Self::Static(_) => HttpMethod::Get,
            Self::Load | Self::Unload | Self::Requests | Self::Cancel(_) => HttpMethod::Post,
        }
    }

    fn allow_header(&self) -> &'static str {
        match self.allowed_method() {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
        }
    }

    fn into_route(self) -> Route {
        match self {
            Self::Capabilities => Route::Capabilities,
            Self::Model => Route::Model,
            Self::Load => Route::Load,
            Self::Unload => Route::Unload,
            Self::Requests => Route::Requests,
            Self::Job(job_id) => Route::Job { job_id },
            Self::Cancel(job_id) => Route::Cancel { job_id },
            Self::Static(manifest_path) => Route::Static { manifest_path },
        }
    }
}

/// Parse one complete HTTP/1.1 request from `bytes`.
///
/// `verified_static_paths` contains manifest-relative paths whose bytes,
/// lengths, hashes, and MIME types were already verified by configuration
/// intake. This function does not decode the request target and does not infer
/// filesystem paths. API-prefix misses never fall through to this set.
pub fn parse_request(
    bytes: &[u8],
    expected_authority: &str,
    verified_static_paths: &[&str],
) -> Result<HttpRequest, HttpAdmissionError> {
    let header_scan_bytes = &bytes[..bytes.len().min(HEADER_MAX_BYTES)];
    let header_end = find_header_end(header_scan_bytes).ok_or_else(|| {
        if bytes.len() > HEADER_MAX_BYTES {
            HttpAdmissionError::bad_request("HTTP headers exceed the configured limit")
        } else {
            HttpAdmissionError::bad_request("incomplete HTTP headers")
        }
    })?;
    let body_start = header_end
        .checked_add(4)
        .ok_or_else(|| HttpAdmissionError::bad_request("HTTP length overflow"))?;
    if body_start > HEADER_MAX_BYTES {
        return Err(HttpAdmissionError::bad_request(
            "HTTP headers exceed the configured limit",
        ));
    }

    let header_text = std::str::from_utf8(&bytes[..header_end])
        .map_err(|_| HttpAdmissionError::bad_request("HTTP headers must be ASCII"))?;
    if !header_text.is_ascii() {
        return Err(HttpAdmissionError::bad_request(
            "HTTP headers must be ASCII",
        ));
    }
    let mut lines = header_text.split("\r\n");
    let request_line = lines
        .next()
        .ok_or_else(|| HttpAdmissionError::bad_request("missing HTTP request line"))?;
    let (method, target) = parse_request_line(request_line)?;
    let headers = parse_headers(lines)?;

    let expected_body_bytes = match headers.content_length {
        Some(length) => length,
        None if method == HttpMethod::Get => 0,
        None => {
            return Err(HttpAdmissionError::bad_request(
                "POST requires one Content-Length header",
            ));
        }
    };
    if expected_body_bytes > BODY_MAX_BYTES {
        return Err(HttpAdmissionError::new(
            ServiceErrorTag::BodyTooLarge,
            413,
            "HTTP body exceeds the configured limit",
        ));
    }
    let request_end = body_start
        .checked_add(expected_body_bytes)
        .ok_or_else(|| HttpAdmissionError::bad_request("HTTP length overflow"))?;
    match bytes.len().cmp(&request_end) {
        std::cmp::Ordering::Less => {
            return Err(HttpAdmissionError::bad_request(
                "HTTP body is shorter than Content-Length",
            ));
        }
        std::cmp::Ordering::Greater => {
            return Err(HttpAdmissionError::bad_request(
                "trailing bytes or request pipelining are rejected",
            ));
        }
        std::cmp::Ordering::Equal => {}
    }
    if method == HttpMethod::Get && expected_body_bytes != 0 {
        return Err(HttpAdmissionError::bad_request(
            "GET requests must not contain a body",
        ));
    }

    match headers.host {
        Some(host) if host == expected_authority => {}
        _ => {
            return Err(HttpAdmissionError::new(
                ServiceErrorTag::HostRejected,
                421,
                "Host does not match the configured loopback authority",
            ));
        }
    }

    enforce_origin(method, headers.origin, expected_authority)?;
    if method == HttpMethod::Post && !headers.content_type.is_some_and(is_json_content_type) {
        return Err(HttpAdmissionError::new(
            ServiceErrorTag::UnsupportedMediaType,
            415,
            "POST requires application/json with optional charset=utf-8",
        ));
    }
    let kind = classify_target(target, verified_static_paths)?;
    if method != kind.allowed_method() {
        return Err(HttpAdmissionError::method_not_allowed(kind.allow_header()));
    }

    Ok(HttpRequest {
        method,
        route: kind.into_route(),
        body: bytes[body_start..request_end].to_vec(),
    })
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

fn parse_request_line(line: &str) -> Result<(HttpMethod, &str), HttpAdmissionError> {
    let mut parts = line.split(' ');
    let method = parts
        .next()
        .ok_or_else(|| HttpAdmissionError::bad_request("invalid HTTP request line"))?;
    let target = parts
        .next()
        .ok_or_else(|| HttpAdmissionError::bad_request("invalid HTTP request line"))?;
    let version = parts
        .next()
        .ok_or_else(|| HttpAdmissionError::bad_request("invalid HTTP request line"))?;
    if parts.next().is_some() || method.is_empty() || target.is_empty() || version != "HTTP/1.1" {
        return Err(HttpAdmissionError::bad_request(
            "request line must use exact HTTP/1.1 origin form",
        ));
    }
    if !target.starts_with('/') || target.bytes().any(|byte| !(0x21..=0x7e).contains(&byte)) {
        return Err(HttpAdmissionError::bad_request(
            "request target must be an ASCII origin-form path",
        ));
    }
    let method = match method {
        "GET" => HttpMethod::Get,
        "POST" => HttpMethod::Post,
        _ => {
            return Err(HttpAdmissionError::method_not_allowed("GET, POST"));
        }
    };
    Ok((method, target))
}

fn parse_headers<'a>(
    lines: impl Iterator<Item = &'a str>,
) -> Result<Headers<'a>, HttpAdmissionError> {
    let mut headers = Headers::default();
    for line in lines {
        if line.is_empty() || line.starts_with(' ') || line.starts_with('\t') {
            return Err(HttpAdmissionError::bad_request(
                "empty or folded HTTP header is rejected",
            ));
        }
        let (name, raw_value) = line
            .split_once(':')
            .ok_or_else(|| HttpAdmissionError::bad_request("invalid HTTP header"))?;
        if name.is_empty() || !name.bytes().all(is_header_name_byte) {
            return Err(HttpAdmissionError::bad_request("invalid HTTP header name"));
        }
        if !raw_value
            .bytes()
            .all(|byte| byte == b'\t' || (0x20..=0x7e).contains(&byte))
        {
            return Err(HttpAdmissionError::bad_request("invalid HTTP header value"));
        }
        let value = raw_value.trim_matches(&[' ', '\t'][..]);

        if name.eq_ignore_ascii_case("host") {
            if headers.host.is_some() {
                return Err(HttpAdmissionError::new(
                    ServiceErrorTag::HostRejected,
                    421,
                    "multiple Host headers are rejected",
                ));
            }
            headers.host = Some(value);
        } else if name.eq_ignore_ascii_case("origin") {
            if headers.origin.is_some() {
                return Err(HttpAdmissionError::new(
                    ServiceErrorTag::OriginRejected,
                    403,
                    "multiple Origin headers are rejected",
                ));
            }
            headers.origin = Some(value);
        } else if name.eq_ignore_ascii_case("content-length") {
            if headers.content_length.is_some() {
                return Err(HttpAdmissionError::bad_request(
                    "multiple Content-Length headers are rejected",
                ));
            }
            if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(HttpAdmissionError::bad_request(
                    "Content-Length must contain decimal digits",
                ));
            }
            headers.content_length = Some(
                value
                    .parse::<usize>()
                    .map_err(|_| HttpAdmissionError::bad_request("Content-Length overflow"))?,
            );
        } else if name.eq_ignore_ascii_case("transfer-encoding") {
            return Err(HttpAdmissionError::bad_request(
                "Transfer-Encoding is not supported",
            ));
        } else if name.eq_ignore_ascii_case("content-type") {
            set_unique(
                &mut headers.content_type,
                value,
                "multiple Content-Type headers are rejected",
            )?;
        }
    }
    Ok(headers)
}

fn set_unique<'a>(
    slot: &mut Option<&'a str>,
    value: &'a str,
    message: &'static str,
) -> Result<(), HttpAdmissionError> {
    if slot.is_some() {
        return Err(HttpAdmissionError::bad_request(message));
    }
    *slot = Some(value);
    Ok(())
}

fn is_header_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}

fn enforce_origin(
    method: HttpMethod,
    actual: Option<&str>,
    expected_authority: &str,
) -> Result<(), HttpAdmissionError> {
    let expected = format!("http://{expected_authority}");
    let accepted = match method {
        HttpMethod::Get => actual.is_none_or(|value| value == expected),
        HttpMethod::Post => actual.is_some_and(|value| value == expected),
    };
    if accepted {
        Ok(())
    } else {
        Err(HttpAdmissionError::new(
            ServiceErrorTag::OriginRejected,
            403,
            "Origin does not match the configured loopback origin",
        ))
    }
}

fn is_json_content_type(value: &str) -> bool {
    let mut parts = value.split(';');
    if !parts
        .next()
        .is_some_and(|media| media.trim().eq_ignore_ascii_case("application/json"))
    {
        return false;
    }
    match (parts.next(), parts.next()) {
        (None, None) => true,
        (Some(parameter), None) => {
            let Some((name, value)) = parameter.split_once('=') else {
                return false;
            };
            name.trim().eq_ignore_ascii_case("charset")
                && value.trim().eq_ignore_ascii_case("utf-8")
        }
        _ => false,
    }
}

fn classify_target(
    target: &str,
    verified_static_paths: &[&str],
) -> Result<TargetKind, HttpAdmissionError> {
    let kind = match target {
        "/uor/v1/workbench/capabilities" => TargetKind::Capabilities,
        "/uor/v1/workbench/model" => TargetKind::Model,
        "/uor/v1/workbench/model/load" => TargetKind::Load,
        "/uor/v1/workbench/model/unload" => TargetKind::Unload,
        "/uor/v1/workbench/requests" => TargetKind::Requests,
        _ => {
            if let Some(remainder) = target.strip_prefix("/uor/v1/workbench/jobs/") {
                if let Some(job_id) = remainder.strip_suffix("/cancel") {
                    if !job_id.contains('/') && is_job_id(job_id) {
                        return Ok(TargetKind::Cancel(job_id.to_owned()));
                    }
                    if !job_id.contains('/') {
                        return Err(HttpAdmissionError::bad_request("invalid job id"));
                    }
                } else if !remainder.contains('/') {
                    if is_job_id(remainder) {
                        return Ok(TargetKind::Job(remainder.to_owned()));
                    }
                    return Err(HttpAdmissionError::bad_request("invalid job id"));
                }
            }

            if target == API_NAMESPACE || target.starts_with("/uor/v1/workbench/") {
                return Err(HttpAdmissionError::not_found());
            }
            let manifest_path = if target == "/" {
                "index.html"
            } else {
                target
                    .strip_prefix('/')
                    .ok_or_else(HttpAdmissionError::not_found)?
            };
            if verified_static_paths
                .iter()
                .any(|candidate| *candidate == manifest_path)
            {
                TargetKind::Static(manifest_path.to_owned())
            } else {
                return Err(HttpAdmissionError::not_found());
            }
        }
    };
    Ok(kind)
}

/// Serialize a JSON response. The caller supplies already serialized UTF-8
/// JSON bytes. API responses are always non-cacheable, close the connection,
/// and intentionally contain no CORS headers.
pub fn serialize_json_response(
    status: u16,
    body: &[u8],
    allow: Option<&'static str>,
) -> Result<Vec<u8>, HttpAdmissionError> {
    serialize_response(status, "application/json; charset=utf-8", body, allow)
}

/// Serialize one verified static asset. `mime` and `body` must come from the
/// accepted asset manifest. The initial shell has stable, non-content-addressed
/// paths, so every asset response is non-cacheable.
pub fn serialize_static_response(mime: &str, body: &[u8]) -> Result<Vec<u8>, HttpAdmissionError> {
    serialize_response(200, mime, body, None)
}

fn serialize_response(
    status: u16,
    content_type: &str,
    body: &[u8],
    allow: Option<&'static str>,
) -> Result<Vec<u8>, HttpAdmissionError> {
    if content_type.is_empty()
        || !content_type
            .bytes()
            .all(|byte| byte == b' ' || (0x21..=0x7e).contains(&byte))
    {
        return Err(HttpAdmissionError::bad_request(
            "response MIME is not a safe HTTP field value",
        ));
    }
    let reason = status_reason(status)
        .ok_or_else(|| HttpAdmissionError::bad_request("unsupported HTTP response status"))?;
    if !matches!(allow, None | Some("GET") | Some("POST") | Some("GET, POST")) {
        return Err(HttpAdmissionError::bad_request(
            "unsupported Allow response value",
        ));
    }
    let mut head = format!(
        "HTTP/1.1 {status} {reason}\r\n\
         Content-Type: {content_type}\r\n\
         Content-Length: {}\r\n\
         Cache-Control: no-store\r\n\
         X-Content-Type-Options: nosniff\r\n\
         Connection: close\r\n",
        body.len()
    );
    if let Some(value) = allow {
        head.push_str("Allow: ");
        head.push_str(value);
        head.push_str("\r\n");
    }
    head.push_str("\r\n");
    let mut response = Vec::with_capacity(head.len().saturating_add(body.len()));
    response.extend_from_slice(head.as_bytes());
    response.extend_from_slice(body);
    Ok(response)
}

fn status_reason(status: u16) -> Option<&'static str> {
    match status {
        200 => Some("OK"),
        202 => Some("Accepted"),
        400 => Some("Bad Request"),
        403 => Some("Forbidden"),
        404 => Some("Not Found"),
        405 => Some("Method Not Allowed"),
        409 => Some("Conflict"),
        413 => Some("Content Too Large"),
        415 => Some("Unsupported Media Type"),
        421 => Some("Misdirected Request"),
        500 => Some("Internal Server Error"),
        503 => Some("Service Unavailable"),
        504 => Some("Gateway Timeout"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AUTHORITY: &str = "127.0.0.1:43177";

    fn static_paths() -> Vec<&'static str> {
        vec!["index.html", "app.js", "styles.css", "NOTICE.txt"]
    }

    fn get(path: &str) -> Vec<u8> {
        format!("GET {path} HTTP/1.1\r\nHost: {AUTHORITY}\r\n\r\n").into_bytes()
    }

    fn post(path: &str, body: &[u8]) -> Vec<u8> {
        let mut request = format!(
            "POST {path} HTTP/1.1\r\nHost: {AUTHORITY}\r\nOrigin: http://{AUTHORITY}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
            body.len()
        )
        .into_bytes();
        request.extend_from_slice(body);
        request
    }

    fn assert_tag(result: Result<HttpRequest, HttpAdmissionError>, tag: ServiceErrorTag) {
        assert_eq!(result.unwrap_err().tag, tag);
    }

    #[test]
    fn classifies_the_exact_seven_public_routes() {
        let assets = static_paths();
        assert_eq!(
            parse_request(&get("/uor/v1/workbench/capabilities"), AUTHORITY, &assets)
                .unwrap()
                .route,
            Route::Capabilities
        );
        assert_eq!(
            parse_request(&get("/uor/v1/workbench/model"), AUTHORITY, &assets)
                .unwrap()
                .route,
            Route::Model
        );
        assert_eq!(
            parse_request(
                &post("/uor/v1/workbench/model/load", b"{}"),
                AUTHORITY,
                &assets,
            )
            .unwrap()
            .route,
            Route::Load
        );
        assert_eq!(
            parse_request(
                &post("/uor/v1/workbench/model/unload", b"{}"),
                AUTHORITY,
                &assets,
            )
            .unwrap()
            .route,
            Route::Unload
        );
        assert_eq!(
            parse_request(
                &post("/uor/v1/workbench/requests", b"{}"),
                AUTHORITY,
                &assets,
            )
            .unwrap()
            .route,
            Route::Requests
        );
        assert_eq!(
            parse_request(&get("/uor/v1/workbench/jobs/42"), AUTHORITY, &assets)
                .unwrap()
                .route,
            Route::Job {
                job_id: "42".to_owned()
            }
        );
        assert_eq!(
            parse_request(
                &post("/uor/v1/workbench/jobs/42/cancel", b"{}"),
                AUTHORITY,
                &assets,
            )
            .unwrap()
            .route,
            Route::Cancel {
                job_id: "42".to_owned()
            }
        );
    }

    #[test]
    fn static_routes_require_exact_verified_membership_without_api_fallback() {
        let mut assets = static_paths();
        assets.push("uor/v1/workbench/compare");
        assert_eq!(
            parse_request(&get("/"), AUTHORITY, &assets).unwrap().route,
            Route::Static {
                manifest_path: "index.html".to_owned()
            }
        );
        assert_eq!(
            parse_request(&get("/app.js"), AUTHORITY, &assets)
                .unwrap()
                .route,
            Route::Static {
                manifest_path: "app.js".to_owned()
            }
        );
        assert_tag(
            parse_request(&get("/missing"), AUTHORITY, &assets),
            ServiceErrorTag::NotFound,
        );
        assert_tag(
            parse_request(&get("/uor/v1/workbench/compare"), AUTHORITY, &assets),
            ServiceErrorTag::NotFound,
        );
        assert_tag(
            parse_request(&get("/app.js?cache=off"), AUTHORITY, &assets),
            ServiceErrorTag::NotFound,
        );
    }

    #[test]
    fn host_and_origin_are_exact() {
        let assets = static_paths();
        let foreign_host = b"GET / HTTP/1.1\r\nHost: localhost:43177\r\n\r\n";
        let foreign_get_origin = format!(
            "GET / HTTP/1.1\r\nHost: {AUTHORITY}\r\nOrigin: http://localhost:43177\r\n\r\n"
        );
        assert_tag(
            parse_request(foreign_host, AUTHORITY, &assets),
            ServiceErrorTag::HostRejected,
        );
        assert_tag(
            parse_request(foreign_get_origin.as_bytes(), AUTHORITY, &assets),
            ServiceErrorTag::OriginRejected,
        );

        let missing_post_origin = format!(
            "POST /uor/v1/workbench/model/load HTTP/1.1\r\nHost: {AUTHORITY}\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{{}}"
        );
        assert_tag(
            parse_request(missing_post_origin.as_bytes(), AUTHORITY, &assets),
            ServiceErrorTag::OriginRejected,
        );
        let null_origin = format!(
            "POST /uor/v1/workbench/model/load HTTP/1.1\r\nHost: {AUTHORITY}\r\nOrigin: null\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{{}}"
        );
        assert_tag(
            parse_request(null_origin.as_bytes(), AUTHORITY, &assets),
            ServiceErrorTag::OriginRejected,
        );
    }

    #[test]
    fn framing_is_unique_exact_and_non_pipelined() {
        let assets = static_paths();
        let exact_body = br#"{"bytes_b64":"AAEC/w=="}"#;
        assert_eq!(
            parse_request(
                &post("/uor/v1/workbench/requests", exact_body),
                AUTHORITY,
                &assets,
            )
            .unwrap()
            .body,
            exact_body.to_vec()
        );
        let duplicate = format!(
            "POST /uor/v1/workbench/requests HTTP/1.1\r\nHost: {AUTHORITY}\r\nOrigin: http://{AUTHORITY}\r\nContent-Type: application/json\r\nContent-Length: 2\r\nContent-Length: 2\r\n\r\n{{}}"
        );
        assert_tag(
            parse_request(duplicate.as_bytes(), AUTHORITY, &assets),
            ServiceErrorTag::BadRequest,
        );
        let transfer = format!(
            "POST /uor/v1/workbench/requests HTTP/1.1\r\nHost: {AUTHORITY}\r\nOrigin: http://{AUTHORITY}\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n"
        );
        assert_tag(
            parse_request(transfer.as_bytes(), AUTHORITY, &assets),
            ServiceErrorTag::BadRequest,
        );
        let mut truncated = post("/uor/v1/workbench/requests", b"{}");
        truncated.pop();
        assert_tag(
            parse_request(&truncated, AUTHORITY, &assets),
            ServiceErrorTag::BadRequest,
        );
        let mut pipelined = get("/");
        pipelined.extend_from_slice(&get("/app.js"));
        assert_tag(
            parse_request(&pipelined, AUTHORITY, &assets),
            ServiceErrorTag::BadRequest,
        );
    }

    #[test]
    fn declared_caps_are_enforced_before_body_copy() {
        let assets = static_paths();
        let oversized_body = format!(
            "POST /uor/v1/workbench/requests HTTP/1.1\r\nHost: {AUTHORITY}\r\nOrigin: http://{AUTHORITY}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
            BODY_MAX_BYTES + 1
        );
        assert_tag(
            parse_request(oversized_body.as_bytes(), AUTHORITY, &assets),
            ServiceErrorTag::BodyTooLarge,
        );
        let long_header = format!(
            "GET / HTTP/1.1\r\nHost: {AUTHORITY}\r\nX-Pad: {}\r\n\r\n",
            "x".repeat(HEADER_MAX_BYTES)
        );
        assert_tag(
            parse_request(long_header.as_bytes(), AUTHORITY, &assets),
            ServiceErrorTag::BadRequest,
        );
    }

    #[test]
    fn post_media_type_is_exact_and_method_errors_are_typed() {
        let assets = static_paths();
        let mut charset = post("/uor/v1/workbench/requests", b"{}");
        let original = b"Content-Type: application/json\r\n";
        let position = charset
            .windows(original.len())
            .position(|window| window == original)
            .unwrap();
        charset.splice(
            position..position + original.len(),
            b"Content-Type: application/json; charset=utf-8\r\n"
                .iter()
                .copied(),
        );
        assert!(parse_request(&charset, AUTHORITY, &assets).is_ok());

        let wrong_media = String::from_utf8(post("/uor/v1/workbench/requests", b"{}"))
            .unwrap()
            .replace("application/json", "text/plain");
        assert_tag(
            parse_request(wrong_media.as_bytes(), AUTHORITY, &assets),
            ServiceErrorTag::UnsupportedMediaType,
        );
        let wrong_method =
            format!("GET /uor/v1/workbench/model/load HTTP/1.1\r\nHost: {AUTHORITY}\r\n\r\n");
        let error = parse_request(wrong_method.as_bytes(), AUTHORITY, &assets).unwrap_err();
        assert_eq!(error.tag, ServiceErrorTag::MethodNotAllowed);
        assert_eq!(error.allow, Some("POST"));

        let post_to_get_without_media = format!(
            "POST /uor/v1/workbench/capabilities HTTP/1.1\r\nHost: {AUTHORITY}\r\nOrigin: http://{AUTHORITY}\r\nContent-Length: 0\r\n\r\n"
        );
        let error =
            parse_request(post_to_get_without_media.as_bytes(), AUTHORITY, &assets).unwrap_err();
        assert_eq!(error.tag, ServiceErrorTag::UnsupportedMediaType);
        assert_eq!(error.allow, None);

        let unknown_without_media = format!(
            "POST /uor/v1/workbench/not-a-route HTTP/1.1\r\nHost: {AUTHORITY}\r\nOrigin: http://{AUTHORITY}\r\nContent-Length: 0\r\n\r\n"
        );
        assert_tag(
            parse_request(unknown_without_media.as_bytes(), AUTHORITY, &assets),
            ServiceErrorTag::UnsupportedMediaType,
        );
    }

    #[test]
    fn malformed_job_ids_never_create_route_aliases() {
        let assets = static_paths();
        for path in [
            "/uor/v1/workbench/jobs/0",
            "/uor/v1/workbench/jobs/01",
            "/uor/v1/workbench/jobs/not-a-number",
        ] {
            assert_tag(
                parse_request(&get(path), AUTHORITY, &assets),
                ServiceErrorTag::BadRequest,
            );
        }
        assert_tag(
            parse_request(
                &get("/uor/v1/workbench/jobs/1/cancel/extra"),
                AUTHORITY,
                &assets,
            ),
            ServiceErrorTag::NotFound,
        );
    }

    #[test]
    fn serializers_close_without_cors_or_fallback_headers() {
        let json = serialize_json_response(405, br#"{"error":"method"}"#, Some("POST")).unwrap();
        let text = String::from_utf8(json).unwrap();
        assert!(text.starts_with("HTTP/1.1 405 Method Not Allowed\r\n"));
        assert!(text.contains("Content-Type: application/json; charset=utf-8\r\n"));
        assert!(text.contains("Cache-Control: no-store\r\n"));
        assert!(text.contains("X-Content-Type-Options: nosniff\r\n"));
        assert!(text.contains("Connection: close\r\n"));
        assert!(text.contains("Allow: POST\r\n"));
        assert!(!text.to_ascii_lowercase().contains("access-control-allow"));

        let asset =
            serialize_static_response("text/javascript; charset=utf-8", b"contents").unwrap();
        let asset = String::from_utf8(asset).unwrap();
        assert!(asset.contains("Cache-Control: no-store\r\n"));
        assert!(serialize_json_response(200, b"{}", Some("GET\r\nInjected: yes")).is_err());
    }
}
