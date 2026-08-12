//! #624 — kappa-registry client: publish compiled artifacts and move κ pins
//! against a Kappa Distribution `/v2/` registry (UOR-Foundation/kappa-registry).
//!
//! Design constraints, from the #624 adoption record:
//!
//! - **Zero shipped-crate dependencies.** This module lives in `xtask`
//!   (build/CI tooling, `publish = false`) and speaks minimal HTTP/1.1 over
//!   `std::net::TcpStream` — no HTTP crate enters any dependency graph.
//! - **The registry is a second witness, not a source of truth.** r4's κ
//!   labels (`blake3:<64 lowercase hex>`, 71 bytes) are byte-identical to the
//!   registry's `KappaLabel` blake3 axis; the registry re-derives the hash on
//!   every PUT, so a successful publish is an out-of-process replay of the
//!   deterministic-rebuild claim for that artifact.
//! - **Pins move only from the value you read** (Gate-E discipline): tag
//!   updates go through `If-Match` compare-and-set; blind overwrites are not
//!   offered by this client at all. Creation uses `If-None-Match: *`.
//! - Trusted-network only: the registry's authorization hook is a stub and
//!   blobs are global content-addressed storage (namespaces are metadata
//!   scoping, NOT an isolation boundary) — recorded on #624.
//!
//! `cargo xtask kappa <publish|fetch|tag-get|tag-cas> …`; the registry base
//! address comes from `R4_KAPPA_REGISTRY` (e.g. `127.0.0.1:8080`).

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

type Fail = Box<dyn std::error::Error>;

/// A parsed HTTP response: status code, the `x-kappa-label` header if any,
/// and the body bytes.
pub struct RegistryResponse {
    pub status: u16,
    pub kappa_label: Option<String>,
    pub body: Vec<u8>,
}

/// One minimal HTTP/1.1 exchange. `Connection: close` so the body is simply
/// everything after the header block (no chunked parsing on our requests:
/// the registry answers our fixed-size exchanges with Content-Length).
fn exchange(
    base: &str,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> Result<RegistryResponse, Fail> {
    let mut stream = TcpStream::connect(base)?;
    stream.set_read_timeout(Some(Duration::from_secs(60)))?;
    stream.set_write_timeout(Some(Duration::from_secs(60)))?;

    let mut req = format!(
        "{method} {path} HTTP/1.1\r\nHost: {base}\r\nConnection: close\r\nContent-Length: {}\r\n",
        body.len()
    );
    for (k, v) in headers {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    req.push_str("\r\n");
    stream.write_all(req.as_bytes())?;
    stream.write_all(body)?;

    let mut raw = Vec::new();
    stream.read_to_end(&mut raw)?;
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or("malformed HTTP response: no header terminator")?;
    let head = std::str::from_utf8(&raw[..split])?;
    let mut lines = head.lines();
    let status_line = lines.next().ok_or("empty HTTP response")?;
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .ok_or("malformed status line")?
        .parse()?;
    let mut kappa_label = None;
    let mut content_length: Option<usize> = None;
    let mut chunked = false;
    for line in lines {
        let Some((k, v)) = line.split_once(':') else {
            continue;
        };
        let (k, v) = (k.trim().to_ascii_lowercase(), v.trim());
        match k.as_str() {
            "x-kappa-label" => kappa_label = Some(v.to_string()),
            "content-length" => content_length = v.parse().ok(),
            "transfer-encoding" if v.eq_ignore_ascii_case("chunked") => chunked = true,
            _ => {}
        }
    }
    let rest = &raw[split + 4..];
    let body = if chunked {
        decode_chunked(rest)?
    } else {
        match content_length {
            Some(n) if n <= rest.len() => rest[..n].to_vec(),
            _ => rest.to_vec(),
        }
    };
    Ok(RegistryResponse {
        status,
        kappa_label,
        body,
    })
}

/// Minimal chunked-transfer decoding (the registry streams some GET bodies).
fn decode_chunked(mut rest: &[u8]) -> Result<Vec<u8>, Fail> {
    let mut out = Vec::new();
    loop {
        let line_end = rest
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or("malformed chunk header")?;
        let size_str = std::str::from_utf8(&rest[..line_end])?;
        let size = usize::from_str_radix(size_str.trim().split(';').next().unwrap_or("0"), 16)?;
        rest = &rest[line_end + 2..];
        if size == 0 {
            return Ok(out);
        }
        if rest.len() < size + 2 {
            return Err("truncated chunk".into());
        }
        out.extend_from_slice(&rest[..size]);
        rest = &rest[size + 2..];
    }
}

/// The r4 κ label of a byte string: `blake3:` + 64 lowercase hex — the same
/// 71-byte form the registry's `KappaLabel` parses and re-derives.
pub fn kappa_of(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3_hex(bytes))
}

/// blake3 via the workspace's pinned implementation. xtask depends on
/// repo-model only, so shell out is avoided by a tiny vendored call path:
/// the `blake3` crate is already in the workspace lock via shipped crates,
/// and xtask may use it as a normal dependency without touching any shipped
/// manifest.
fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// PUT an artifact by its κ. The registry independently re-derives the label
/// over the body and refuses a mismatch — the second-witness property.
pub fn publish(base: &str, ns: &str, artifact: &Path) -> Result<String, Fail> {
    let bytes = std::fs::read(artifact)?;
    let kappa = kappa_of(&bytes);
    let resp = exchange(
        base,
        "PUT",
        &format!("/v2/{ns}/blobs/{kappa}"),
        &[("Content-Type", "application/octet-stream")],
        &bytes,
    )?;
    match resp.status {
        201 | 200 => Ok(kappa),
        409 => Err(format!("registry refused {kappa}: label/content mismatch (409)").into()),
        s => Err(format!("publish {kappa} failed: HTTP {s}").into()),
    }
}

/// GET an artifact by κ and verify the bytes re-derive the label locally —
/// never trust, always re-derive (the registry does the same on its side).
pub fn fetch_verified(base: &str, ns: &str, kappa: &str) -> Result<Vec<u8>, Fail> {
    let resp = exchange(base, "GET", &format!("/v2/{ns}/blobs/{kappa}"), &[], &[])?;
    if resp.status != 200 {
        return Err(format!("fetch {kappa} failed: HTTP {}", resp.status).into());
    }
    let derived = kappa_of(&resp.body);
    if derived != kappa {
        return Err(format!("fetch {kappa}: body re-derives to {derived} — corrupt").into());
    }
    Ok(resp.body)
}

/// Read a tag's current κ, or None if the tag does not exist.
pub fn tag_get(base: &str, ns: &str, name: &str) -> Result<Option<String>, Fail> {
    let resp = exchange(
        base,
        "GET",
        &format!("/v2/{ns}/tags/{name}?raw=true"),
        &[],
        &[],
    )?;
    match resp.status {
        200 => {
            let v = resp
                .kappa_label
                .unwrap_or_else(|| String::from_utf8_lossy(&resp.body).trim().to_string());
            Ok(Some(v))
        }
        404 => Ok(None),
        s => Err(format!("tag get {name} failed: HTTP {s}").into()),
    }
}

/// Move a κ pin with compare-and-set semantics (Gate-E: a pin only moves from
/// the value you read). `expected = None` requires the tag NOT to exist yet
/// (`If-None-Match: *`); `Some(old)` requires it to currently equal `old`
/// (`If-Match`). A stale expectation is a hard error, never an overwrite.
pub fn tag_cas(
    base: &str,
    ns: &str,
    name: &str,
    kappa: &str,
    expected: Option<&str>,
) -> Result<(), Fail> {
    let path = format!("/v2/{ns}/tags/{name}?kappa={kappa}");
    let resp = match expected {
        Some(old) => exchange(base, "PUT", &path, &[("If-Match", old)], &[])?,
        None => exchange(base, "PUT", &path, &[("If-None-Match", "*")], &[])?,
    };
    match resp.status {
        200 | 201 => Ok(()),
        409 => Err(format!(
            "tag {name}: compare-and-set refused (pin moved since it was read) — re-read and \
             re-decide, never force"
        )
        .into()),
        404 => Err(format!("tag {name}: content {kappa} not in store — publish first").into()),
        s => Err(format!("tag cas {name} failed: HTTP {s}").into()),
    }
}

/// `cargo xtask kappa <cmd> …` entry point.
pub fn run(args: &[String]) -> Result<(), Fail> {
    let base = std::env::var("R4_KAPPA_REGISTRY")
        .map_err(|_| "set R4_KAPPA_REGISTRY (e.g. 127.0.0.1:8080)")?;
    match args {
        [cmd, ns, artifact] if cmd == "publish" => {
            let kappa = publish(&base, ns, Path::new(artifact))?;
            let bytes = fetch_verified(&base, ns, &kappa)?;
            println!(
                "published + round-trip verified: {kappa} ({} bytes) in namespace {ns}",
                bytes.len()
            );
            Ok(())
        }
        [cmd, ns, kappa, out] if cmd == "fetch" => {
            let bytes = fetch_verified(&base, ns, kappa)?;
            std::fs::write(out, &bytes)?;
            println!(
                "fetched + re-derived {kappa} -> {out} ({} bytes)",
                bytes.len()
            );
            Ok(())
        }
        [cmd, ns, name] if cmd == "tag-get" => {
            match tag_get(&base, ns, name)? {
                Some(k) => println!("{k}"),
                None => println!("(unset)"),
            }
            Ok(())
        }
        [cmd, ns, name, kappa, rest @ ..] if cmd == "tag-cas" => {
            let expected = rest.first().map(|s| s.as_str());
            tag_cas(&base, ns, name, kappa, expected)?;
            println!(
                "pin {name} -> {kappa} ({})",
                match expected {
                    Some(old) => format!("moved from {old}"),
                    None => "created".to_string(),
                }
            );
            Ok(())
        }
        _ => Err(
            "cargo xtask kappa <publish ns file | fetch ns kappa out | tag-get ns name | \
                  tag-cas ns name kappa [expected]>"
                .into(),
        ),
    }
}
